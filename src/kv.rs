#![deny(missing_docs)]
//! this is a crate doc
use crate::error::{KvsError::*, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    path::PathBuf,
};
/// there is just a warpper for a HashMap.
pub struct KvStore {
    index: HashMap<String, u64>,
    disk_db: File,
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
        let disk_db = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path)?;
        let mut reader = BufReader::new(disk_db);
        let mut index = HashMap::new();
        let mut offset = reader.stream_position()?;
        let mut buf = String::new();
        while let Ok(len) = reader.read_line(&mut buf) {
            if len > 0 {
                match ron::de::from_str(&buf)? {
                    Log::Set(k, _) => index.insert(k, offset),
                    Log::Rm(k) => index.remove(&k),
                };
                offset += len as u64;
                buf.clear();
            } else {
                break;
            }
        }
        Ok(Self {
            index,
            disk_db: reader.into_inner(),
        })
    }
    /// Set the value of a string key to a string.
    /// Return an error if the value is not written successfully.    
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let set_log = Log::Set(key.to_owned(), value.to_owned());
        let mut buf = ron::ser::to_string(&set_log)?;
        buf.push('\n');
        let offset = self.disk_db.seek(SeekFrom::End(0))?;
        self.disk_db.write(buf.as_bytes())?;
        self.index.insert(key, offset);
        Ok(())
    }
    /// Get the string value of a string key.
    /// If the key does not exist, return `None`.
    /// Return an error if the value is not read successfully.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        let v = self.index.get(&key);
        if let Some(&pos) = v {
            let mut reader = BufReader::new(&self.disk_db);
            reader.seek(SeekFrom::Start(pos))?;
            let mut buf = String::new();
            reader.read_line(&mut buf)?;
            match ron::from_str(&buf)? {
                Log::Rm(_) => Err(Inner.into()),
                Log::Set(k, v) => {
                    if k.eq(&key) {
                        Ok(Some(v))
                    } else {
                        Err(Inner.into())
                    }
                }
            }
        } else {
            Ok(None)
        }
        // Ok(self.index.get(&key).and_then(|f| Some(f.to_owned())))
    }
    /// Remove a given key.
    /// Return an error if the key does not exist or is not removed successfully.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let mut rm_log = ron::ser::to_string(&Log::Rm(key.to_owned()))?;
            rm_log.push('\n');
            self.disk_db.write(rm_log.as_bytes())?;
            self.index.remove(&key);
            Ok(())
        } else {
            Err(KeyNotFound(key).into())
        }
    }
}
