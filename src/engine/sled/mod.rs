use super::Result;
use crate::KvsEngine;
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
        self.0
            .get(key)?
            .map(|v| String::from_utf8(v.as_ref().to_vec()))
            .transpose()
            .map_err(|e| e.into())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        self.0.remove(key)?;
        self.0.flush()?;
        Ok(())
    }
}
