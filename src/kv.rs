#![deny(missing_docs)]
//! this is a crate doc
use crate::error::{Result, *};
use crate::seekablefile::SeekableLSFile;
use serde::{Deserialize, Serialize};
use std::fs::rename;
use std::io::SeekFrom;
use std::{collections::HashMap, path::PathBuf};
/// there is just a warpper for a HashMap.
pub struct KvStore {
    index: HashMap<String, u64>,
    path: PathBuf,
    disk_db: SeekableLSFile,
    invalid_count: u64,
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
        let mut path = path.into();
        path.push("mydb");
        let mut disk_db = SeekableLSFile::new(&path)?;
        let mut index = HashMap::new();
        let mut invalid_count = 0;
        let mut offset = 0;
        loop {
            let mut buf = String::new();
            let l = disk_db.read_line_from(SeekFrom::Start(offset), &mut buf)?;
            if l == 0 {
                break;
            }
            match ron::de::from_str(&buf)? {
                Log::Set(k, _) => {
                    if index.insert(k, offset).is_some() {
                        invalid_count += 1;
                    }
                }
                Log::Rm(k) => {
                    index.remove(&k);
                    invalid_count += 1;
                }
            };
            offset += l as u64;
        }
        Ok(Self {
            index,
            path,
            disk_db,
            invalid_count,
        })
    }
    /// Set the value of a string key to a string.
    /// Return an error if the value is not written successfully.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let set_log = Log::Set(key.to_owned(), value);
        let mut buf = ron::ser::to_string(&set_log)?;
        buf.push('\n');
        let (offset, _) = self.disk_db.append(buf.as_bytes())?;
        if self.index.insert(key, offset).is_some() {
            self.invalid_count += 1;
            self.compaction_trigger()?;
        };
        Ok(())
    }

    /// Get the string value of a string key.
    /// If the key does not exist, return `None`.
    /// Return an error if the value is not read successfully.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(&pos) = self.index.get(&key) {
            match inner_get(&mut self.disk_db, pos)? {
                Log::Rm(_) => Err(KvsError::Inner.into()),
                Log::Set(k, v) => {
                    if k.eq(&key) {
                        Ok(Some(v))
                    } else {
                        Err(KvsError::Inner.into())
                    }
                }
            }
        } else {
            Ok(None)
        }
    }
    /// Remove a given key.
    /// Return an error if the key does not exist or is not removed successfully.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let mut rm_log = ron::ser::to_string(&Log::Rm(key.to_owned()))?;
            rm_log.push('\n');
            self.disk_db.append(rm_log.as_bytes())?;
            self.index.remove(&key);
            self.invalid_count += 1;
            self.compaction_trigger()?;
            Ok(())
        } else {
            Err(KvsError::KeyNotFound(key).into())
        }
    }

    fn compaction_trigger(&mut self) -> Result<()> {
        if self.invalid_count >= self.index.len() as u64 {
            self.compaction()
        } else {
            Ok(())
        }
    }

    fn compaction(&mut self) -> Result<()> {
        let mut file_name = self
            .path
            .file_name()
            .map_or("mydb".into(), |s| s.to_string_lossy())
            .to_string();
        file_name.push_str(".new");
        let mut new_path = self.path.clone();
        new_path.set_file_name(file_name);

        let mut new_db = SeekableLSFile::new(&new_path)?;
        let Self { index, disk_db, .. } = self;
        for pos in index.values_mut() {
            let log = inner_get(disk_db, *pos)?;
            let mut buf = ron::ser::to_string(&log)?;
            buf.push('\n');
            let (tmp, _) = new_db.append(buf.as_bytes())?;
            *pos = tmp;
        }
        rename(&new_path, &self.path)?;
        self.path = new_path;
        self.disk_db = new_db;
        self.invalid_count = 0;
        Ok(())
    }
}

fn inner_get(disk_db: &mut SeekableLSFile, pos: u64) -> Result<Log> {
    let mut buf = String::new();
    disk_db.read_line_from(SeekFrom::Start(pos), &mut buf)?;
    Ok(ron::from_str(&buf)?)
}
