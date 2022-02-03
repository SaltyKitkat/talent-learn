use sled::Db;

use crate::error::Result;
use crate::KvsEngine;
use std::path::Path;

pub struct SledKvsEngine(Db);
impl SledKvsEngine {
    pub fn open(p: impl AsRef<Path>) -> Result<Self> {
        Ok(Self(sled::open(p)?))
    }
}
impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        todo!()
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        todo!()
    }

    fn remove(&mut self, key: String) -> Result<()> {
        todo!()
    }
}
