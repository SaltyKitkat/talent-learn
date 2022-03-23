#![deny(missing_docs)]
//! this is a crate doc
use super::Result;
use crate::{
    error::{KvsError, KvsResult},
    KvsEngine,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, remove_file, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    mem,
    path::{Path, PathBuf},
};

/// when the invalid size is larger than `COMPACTION_THRESHOLD`(in bytes), a compaction process will be triggered.
const COMPACTION_THRESHOLD: usize = 4 * 1024 * 1024;

type FileId = u32;
type Key = String;
type Value = String;
type Index = HashMap<Key, LogMeta>;

/// KvStore
/// the main struct of KVS
pub struct KvStore {
    index: Index,
    path: PathBuf,
    files: HashMap<FileId, File>,
    write_id: FileId,
    uncompact_size: usize,
}

impl KvStore {
    /// open a KvStore instance from the given path.
    /// the path should be a dir with some .kvs file in it.
    /// this function will use the kvs files to build a KvStore db and return it if succeed.
    /// If any error met, this function will return it.
    pub fn open(path: impl AsRef<Path>) -> KvsResult<Self> {
        fn load(index: &mut Index, file_id: FileId, db_file: &mut File) -> KvsResult<usize> {
            let mut uncompact_size = 0;
            let mut offset = db_file.seek(SeekFrom::Start(0))?;
            let mut t = serde_json::Deserializer::from_reader(db_file).into_iter::<Log>();
            while let Some(cmd) = t.next() {
                let new_offset = t.byte_offset() as u64;
                match cmd? {
                    Log { key, value: None } => {
                        uncompact_size += index.remove(&key).map(|l| l.len).unwrap()
                    }
                    Log { key, value: _ } => {
                        uncompact_size += index
                            .insert(
                                key,
                                LogMeta {
                                    file_id,
                                    offset,
                                    len: (new_offset - offset) as usize,
                                },
                            )
                            .map(|l| l.len)
                            .unwrap_or(0);
                    }
                }
                offset = new_offset;
            }
            Ok(uncompact_size)
        }

        let path = path.as_ref();
        fs::create_dir_all(path)?;
        let log_list = fs::read_dir(path)?;
        let log_list = {
            let mut log_list = log_list
                .flat_map(|f| -> Result<_> { Ok(f?.path()) })
                .filter(|p| p.extension() == Some("kvs".as_ref()))
                .flat_map(|p| p.file_stem().and_then(OsStr::to_str).map(str::parse))
                .flatten()
                .collect::<Vec<FileId>>();
            log_list.sort_unstable();
            log_list
        };
        let mut files = HashMap::new();
        let mut uncompact_size = 0;
        let mut index = Index::new();
        for &i in log_list.iter() {
            let mut file = open_ro(path, i)?;
            uncompact_size += load(&mut index, i, &mut file)?;
            files.insert(i, file);
        }
        let write_id = log_list.last().unwrap_or(&0) + 1;
        let write_path = get_path(path, write_id);
        let write_file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .read(true)
            .open(write_path)?;
        // let writer = LogWriter::new(new_file_id, new_file.try_clone()?);
        files.insert(write_id, write_file);
        Ok(Self {
            index,
            path: path.to_path_buf(),
            files,
            write_id,
            uncompact_size,
        })
    }

    fn compaction_trigger(&mut self) -> KvsResult<()> {
        if self.uncompact_size >= COMPACTION_THRESHOLD {
            self.compaction_inner().map_err(|e| {
                if let KvsError::Inner(s) = e {
                    KvsError::CompactionError(s)
                } else {
                    panic!("unexpect error")
                }
            })
        } else {
            Ok(())
        }
    }

    // note: index and writer are replaced by the new ones while readers are just cleared.
    fn compaction_inner(&mut self) -> KvsResult<()> {
        let new_write_id = self.write_id.wrapping_add(1);
        let mut new_write_file = open_rw(&self.path, new_write_id)?;
        let old_index = mem::take(&mut self.index);
        for (key, meta) in old_index.into_iter() {
            let log = read_log(self.files.get_mut(&meta.file_id).unwrap(), &meta)?;
            debug_assert!(log.value.is_some());
            let (offset, len) = write_log(&mut new_write_file, &log)?;
            let new_meta = LogMeta {
                file_id: new_write_id,
                offset,
                len,
            };
            self.index.insert(key, new_meta);
        }
        let new_files = {
            let mut new_files = HashMap::new();
            new_files.insert(new_write_id, new_write_file);
            new_files
        };
        let old_files = mem::replace(&mut self.files, new_files);
        for &to_rm_id in old_files.keys() {
            remove_file(get_path(&self.path, to_rm_id))?;
        }
        self.write_id = new_write_id;
        self.uncompact_size = 0;
        Ok(())
    }
}
impl KvsEngine for KvStore {
    /// Set the value of a key.
    /// Return `Ok(())` if succeed.
    /// Return an error if the value is not set successfully.
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let log = Log {
            key,
            value: Some(value),
        };
        let (offset, len) = write_log(self.files.get_mut(&self.write_id).unwrap(), &log)?;
        self.uncompact_size += self
            .index
            .insert(
                log.key,
                LogMeta {
                    file_id: self.write_id,
                    offset,
                    len,
                },
            )
            .map(|l| l.len)
            .unwrap_or(0);
        self.compaction_trigger()?;
        Ok(())
    }

    /// Get the value of a key.
    /// Return `Ok(Some(value))` if something is found.
    /// If the key does not exist, return `Ok(None)`.
    /// Return an error if the value is not read successfully.
    fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.index.get(&key) {
            Some(meta) => {
                let log = read_log(self.files.get_mut(&meta.file_id).unwrap(), meta)?;
                Ok(log.value)
            }
            None => Ok(None),
        }
    }

    /// Remove a given key.
    /// Return an error if the key does not exist or is not removed successfully.
    fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let log = Log { key, value: None };
            let (_offset, len) = write_log(self.files.get_mut(&self.write_id).unwrap(), &log)?;
            self.uncompact_size += self.index.remove(&log.key).unwrap().len + len;
            self.compaction_trigger()?;
            Ok(())
        } else {
            Err(KvsError::KeyNotFound { key })
        }
    }
}

impl Drop for KvStore {
    fn drop(&mut self) {
        self.compaction_inner().ok();
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Log {
    pub key: Key,
    pub value: Option<Value>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct LogMeta {
    pub file_id: FileId,
    pub offset: u64,
    pub len: usize,
}

fn get_path(path: &Path, id: FileId) -> PathBuf {
    path.join(format!("{id}.kvs"))
}

fn open_ro(path: &Path, id: FileId) -> KvsResult<File> {
    let db_path = get_path(path, id);
    let db_file = OpenOptions::new().read(true).open(db_path)?;
    Ok(db_file)
}

fn open_rw(path: &Path, id: FileId) -> KvsResult<File> {
    let db_path = get_path(path, id);
    let db_file = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(db_path)?;
    Ok(db_file)
}

fn read_log(file: &mut File, meta: &LogMeta) -> KvsResult<Log> {
    file.seek(SeekFrom::Start(meta.offset))?;
    let mut buf = vec![0; meta.len];
    file.read_exact(&mut buf)?;
    serde_json::de::from_slice(&buf).map_err(|e| e.into())
}

fn write_log(file: &mut File, log: &Log) -> KvsResult<(u64, usize)> {
    let buf = serde_json::ser::to_vec(log)?;
    let old_offset = file.seek(SeekFrom::End(0))?;
    file.write_all(&buf)?;
    file.flush()?;
    Ok((old_offset, buf.len()))
}
