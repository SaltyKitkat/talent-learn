use super::Result;
use crate::{error::KvsError, KvsEngine};
use sled::Db;
use std::path::Path;

pub struct SledKvsEngine(Db);
impl SledKvsEngine {
    pub fn open(p: impl AsRef<Path>) -> Result<Self> {
        Ok(Self(sled::open(p)?))
    }
}
impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.0.insert(key, value.into_bytes())?;
        self.0.flush()?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        Ok(self
            .0
            .get(key)?
            .map(|v| String::from_utf8_lossy(v.as_ref()).to_string()))
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let ret = self
            .0
            .remove(&key)?
            .and(Some(()))
            .ok_or(KvsError::KeyNotFound { key });
        self.0.flush()?;
        ret
    }
}
