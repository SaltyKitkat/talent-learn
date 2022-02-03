#![deny(missing_docs)]
//! this is a crate doc
use crate::{
    error::{KvsError, Result},
    KvsEngine,
};
use logmisc::{LogMeta, LogReader, LogReaders, LogWriter};
use std::{
    ffi::OsStr,
    fs::{self, remove_file, File, OpenOptions},
    io::{Seek, SeekFrom},
    ops::DerefMut,
    path::{Path, PathBuf},
};

/// when the invalid size is larger than `COMPACTION_THRESHOLD`(in bytes), a compaction process will be triggered.
const COMPACTION_THRESHOLD: usize = 4 * 1024 * 1024;

mod kvsindex;
mod logmisc;
use self::logmisc::Log;
use kvsindex::KvsIndex;

/// KvStore
/// the main struct of KVS
pub struct KvStore {
    index: KvsIndex,
    path: PathBuf,
    readers: LogReaders,
    writer: LogWriter<File>,
    uncompact_size: usize,
}

impl KvStore {
    /// open a KvStore instance from the given path.
    /// the path should be a dir with some .kvs file in it.
    /// this function will use the kvs files to build a KvStore db and return it if succeed.
    /// If any error met, this function will return it.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        fs::create_dir_all(path)?;
        let log_list = fs::read_dir(path)?;
        let log_list = {
            let mut log_list = log_list
                .flat_map(|f| -> Result<_> { Ok(f?.path()) })
                .filter(|p| p.extension() == Some("kvs".as_ref()))
                .flat_map(|p| p.file_stem().and_then(OsStr::to_str).map(str::parse))
                .flatten()
                .collect::<Vec<u64>>();
            log_list.sort_unstable();
            log_list
        };
        let mut readers = LogReaders::new();
        let mut uncompact_size = 0;
        let mut index = KvsIndex::new();
        for &i in log_list.iter() {
            let mut reader = db_open(path, i)?;
            uncompact_size += load(&mut index, i, &mut reader)?;
            readers.insert(i, reader);
        }
        let new_file_id = log_list.last().unwrap_or(&0) + 1;
        let mut new_file_path = path.to_owned();
        new_file_path.push(format!("{new_file_id}.kvs"));
        let new_file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .read(true)
            .open(new_file_path)?;
        let writer = LogWriter::new(new_file_id, new_file.try_clone()?)?;
        readers.insert(new_file_id, LogReader::new(new_file));
        Ok(Self {
            index,
            path: path.to_path_buf(),
            readers,
            writer,
            uncompact_size,
        })
    }

    fn compaction_trigger(&mut self) -> Result<()> {
        if self.uncompact_size >= COMPACTION_THRESHOLD {
            self.compaction_inner()
        } else {
            Ok(())
        }
    }

    // note: index and writer are replaced by the new ones while readers is just cleared.
    fn compaction_inner(&mut self) -> Result<()> {
        let mut new_path = self.path.to_owned();
        let new_file_id = 0;
        new_path.push(new_file_id.to_string() + ".kvs");
        let new_file = OpenOptions::new()
            .create_new(true)
            .read(true)
            .append(true)
            .open(new_path)?;
        let new_reader = LogReader::new(new_file.try_clone()?);
        let mut new_writer = LogWriter::new(new_file_id, new_file)?;
        let mut new_index = KvsIndex::new();
        for (key, cmd) in self.index.iter() {
            let log = self.readers.read_log(cmd)?;
            debug_assert!(matches!(log, Log::Set(..)));
            let (read_key, m) = new_writer.append_log(log)?;
            debug_assert_eq!(&read_key, key);
            let old_len = new_index.insert(read_key, m);
            debug_assert_eq!(old_len, 0);
        }
        self.index = new_index;
        self.writer = new_writer;
        for &i in self.readers.keys() {
            let mut old_file_path = self.path.to_owned();
            old_file_path.push(i.to_string() + ".kvs");
            remove_file(old_file_path)?;
        }
        self.readers.clear();
        self.readers.insert(new_file_id, new_reader);
        self.uncompact_size = 0;
        Ok(())
    }
}

impl KvsEngine for KvStore {
    /// Set the value of a key.
    /// Return `Ok(())` if succeed.
    /// Return an error if the value is not set successfully.
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let (k, m) = self.writer.append_log(Log::Set(key, value))?;
        self.uncompact_size += self.index.insert(k, m);
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
                let read_log = self.readers.read_log(meta)?;
                debug_assert!(matches!(read_log, Log::Set(..)));
                match read_log {
                    Log::Set(_, s) => Ok(Some(s)),
                    Log::Rm(..) => unreachable!(),
                }
            }
            None => Ok(None),
        }
    }

    /// Remove a given key.
    /// Return an error if the key does not exist or is not removed successfully.
    fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let (key, cmd) = self.writer.append_log(Log::Rm(key))?;
            self.uncompact_size += self.index.remove(&key)? + cmd.len();
            self.compaction_trigger()?;
            Ok(())
        } else {
            Err(KvsError::KeyNotFound(key).into())
        }
    }
}

impl Drop for KvStore {
    fn drop(&mut self) {
        self.compaction_inner();
    }
}

fn load(index: &mut KvsIndex, file_id: u64, db_file: &mut LogReader<File>) -> Result<usize> {
    let mut invalid_size = 0;
    let mut pos = db_file.seek(SeekFrom::Start(0))?;
    let mut t = serde_json::Deserializer::from_reader(db_file.deref_mut()).into_iter::<Log>();
    while let Some(cmd) = t.next() {
        let new_pos = t.byte_offset() as u64;
        match cmd? {
            Log::Rm(k) => invalid_size += index.remove(&k)?,
            Log::Set(k, _) => {
                invalid_size +=
                    index.insert(k, LogMeta::new(file_id, pos, (new_pos - pos) as usize))
            }
        }
        pos = new_pos;
    }
    Ok(invalid_size)
}

fn db_open(path: &Path, i: u64) -> Result<LogReader<File>> {
    let mut db_path = path.to_owned();
    db_path.push(i.to_string() + ".kvs");
    let db_file = OpenOptions::new().read(true).open(db_path)?;
    Ok(LogReader::new(db_file))
}
