#![deny(missing_docs)]
//! this is a crate doc
use crate::error::{KvsError, Result};
use crate::seekablefile::{PosBufReader, PosBufWriter};
use failure::ResultExt;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env::current_dir;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};

use std::path::Path;
use std::{collections::HashMap, path::PathBuf};
use std::{u64, usize};

const COMPACTION_THRESHOLD: usize = 4 * 1024 * 1024;

struct DbCmdHandle {
    file_id: u64,
    offset: u64,
    len: usize,
}
struct KvsIndex(HashMap<String, DbCmdHandle>);
impl KvsIndex {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn insert(&mut self, key: String, index: DbCmdHandle) -> usize {
        match self.0.insert(key, index) {
            Some(DbCmdHandle { len, .. }) => len,
            None => 0,
        }
    }

    fn remove(&mut self, key: &str) -> Result<usize> {
        match self.0.remove(key) {
            Some(DbCmdHandle { len, .. }) => Ok(len),
            None => Err(KvsError::Inner(String::from("Failed to find the key to remove")).into()),
        }
    }
    // fn get(&self, key: &str) -> Option<&DbCmdHandle> {
    //     self.0.get(key)
    // }
}
impl Deref for KvsIndex {
    type Target = HashMap<String, DbCmdHandle>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
/// KvStore
/// the main struct of KVS
pub struct KvStore {
    index: KvsIndex,
    path: PathBuf,
    readers: BTreeMap<u64, PosBufReader<File>>,
    writer: PosBufWriter<File>,
    new_file_id: u64,
    invalid_size: usize,
}

impl Drop for KvStore {
    fn drop(&mut self) {
        if let Ok(f) = current_dir() {
            if let Ok(f) = OpenOptions::new().create(true).write(true).open(f) {
                let mut f = BufWriter::new(f);
                for &i in self.readers.keys() {
                    writeln!(f, "{}", i); // todo: handle the failed situation
                }
            }
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
enum Log {
    Set(String, String),
    Rm(String),
}

impl KvStore {
    /// open a KvStore instance from the given path.
    /// return the KvStore
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path: PathBuf = path.into();
        if !(path.is_dir()) {
            return Err(KvsError::IO).context("Working dir not found")?; // todo: create dir?
        }
        let mut log_list = path
            .read_dir()?
            .flat_map(|f| -> Result<_> { Ok(f?.path()) })
            .filter(|p| p.extension() == Some("kvs".as_ref()))
            .flat_map(|p| p.file_stem().and_then(OsStr::to_str).map(str::parse))
            .flatten()
            .collect::<Vec<u64>>();
        log_list.sort_unstable();
        let mut readers = BTreeMap::new();
        let mut invalid_size = 0;
        let mut index = KvsIndex::new();
        for &i in log_list.iter() {
            let mut reader = db_open(&path, i)?;
            invalid_size += load(&mut index, i, &mut reader)?;
            readers.insert(i, reader);
        }
        let new_file_id = log_list.last().unwrap_or(&0) + 1;
        let mut new_file_path = path.to_owned();
        new_file_path.push(new_file_id.to_string() + ".kvs");
        let new_file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .read(true)
            .open(new_file_path)?;
        let writer = PosBufWriter::new(new_file.try_clone()?)?;
        readers.insert(new_file_id, PosBufReader::new(new_file));
        Ok(Self {
            index,
            path,
            readers,
            writer,
            new_file_id,
            invalid_size,
        })
    }
    /// Set the value of a string key to a string.
    /// Return an error if the value is not written successfully.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let KvStore {
            index,
            writer,
            new_file_id,
            invalid_size,
            ..
        } = self;
        let set_log = Log::Set(key.to_owned(), value);
        let buf = serde_json::ser::to_vec(&set_log)?;
        let offset = writer.append(&buf)?;
        self.writer.flush()?;
        *invalid_size += index.insert(
            key,
            DbCmdHandle {
                file_id: *new_file_id,
                offset,
                len: buf.len(),
            },
        );
        self.compaction_trigger()?;
        Ok(())
    }

    /// Get the string value of a string key.
    /// If the key does not exist, return `None`.
    /// Return an error if the value is not read successfully.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.index.get(&key) {
            Some(DbCmdHandle {
                file_id,
                offset: pos,
                len,
            }) => {
                if let Some(reader) = self.readers.get_mut(file_id) {
                    let mut buf = vec![0; *len];
                    reader.read_at(*pos, &mut buf)?;
                    if let Log::Set(k, v) = serde_json::de::from_slice(&buf)? {
                        if k.eq(&key) {
                            return Ok(Some(v));
                        } else {
                            return Err(KvsError::Inner(String::from(
                                "the key read from disk is different from the key in index",
                            ))
                            .into());
                        }
                    }
                }
                Err(KvsError::Inner(String::from("failed to get file_id in index")).into())
            }
            None => Ok(None),
        }
    }

    /// Remove a given key.
    /// Return an error if the key does not exist or is not removed successfully.
    pub fn remove(&mut self, key: String) -> Result<()> {
        let Self {
            index,
            writer,
            invalid_size,
            ..
        } = self;
        if index.contains_key(&key) {
            let log = serde_json::ser::to_vec(&Log::Rm(key.to_owned()))?;
            writer.append(&log)?;
            writer.flush()?;
            *invalid_size += index.remove(&key)? + log.len();
            self.compaction_trigger()?;
            Ok(())
        } else {
            Err(KvsError::KeyNotFound(key).into())
        }
    }

    fn compaction_trigger(&mut self) -> Result<()> {
        if self.invalid_size >= COMPACTION_THRESHOLD {
            self.compaction_inner()
        } else {
            Ok(())
        }
    }

    fn compaction_inner(&mut self) -> Result<()> {
        let Self {
            index,
            path,
            readers,
            writer,
            new_file_id,
            invalid_size,
        } = self;
        todo!()
        //         let mut file_name = self
        //             .path
        //             .file_name()
        //             .map_or("mydb".into(), |s| s.to_string_lossy())
        //             .to_string();
        //         file_name.push_str(".new");
        //         let mut new_path = self.path.clone();
        //         new_path.set_file_name(file_name);

        //         let mut new_db = SeekableLSFile::new(&new_path)?;
        //         let Self { index, disk_db, .. } = self;
        //         for pos in index.values_mut() {
        //             let log = inner_get(disk_db, *pos)?;
        //             let mut buf = ron::ser::to_string(&log)?;
        //             buf.push('\n');
        //             let (tmp, _) = new_db.append(buf.as_bytes())?;
        //             *pos = tmp;
        //         }
        //         rename(&new_path, &self.path)?;
        //         self.path = new_path;
        //         self.disk_db = new_db;
        //         self.invalid_count = 0;
        //         Ok(())
    }
}

fn load(index: &mut KvsIndex, file_id: u64, db_file: &mut PosBufReader<File>) -> Result<usize> {
    let mut invalid_size = 0;
    let mut pos = db_file.seek(SeekFrom::Start(0))?;
    let mut t = serde_json::Deserializer::from_reader(db_file.deref_mut()).into_iter::<Log>();
    while let Some(cmd) = t.next() {
        let new_pos = t.byte_offset() as u64;
        match cmd? {
            Log::Rm(k) => invalid_size += index.remove(&k)?,
            Log::Set(k, _) => {
                invalid_size += index.insert(
                    k,
                    DbCmdHandle {
                        file_id,
                        offset: pos,
                        len: (new_pos - pos) as usize,
                    },
                )
            }
        }
        pos = new_pos;
    }
    Ok(invalid_size)
}
fn db_open(path: &Path, i: u64) -> Result<PosBufReader<File>> {
    let mut db_path = path.to_owned();
    db_path.push(i.to_string() + ".kvs");
    let db_file = OpenOptions::new().read(true).open(db_path)?;
    Ok(PosBufReader::new(db_file))
}
