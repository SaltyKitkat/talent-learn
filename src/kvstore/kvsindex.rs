use super::logmisc::LogMeta;
use crate::error::{KvsError, Result};
use std::{collections::HashMap, ops::Deref};

pub(crate) struct KvsIndex(HashMap<String, LogMeta>);
impl KvsIndex {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn insert(&mut self, key: String, index: LogMeta) -> usize {
        match self.0.insert(key, index) {
            Some(cmd) => cmd.len(),
            None => 0,
        }
    }

    pub(crate) fn remove(&mut self, key: &str) -> Result<usize> {
        match self.0.remove(key) {
            Some(cmd) => Ok(cmd.len()),
            None => Err(KvsError::Inner(String::from("Failed to find the key to remove")).into()),
        }
    }
    // fn get(&self, key: &str) -> Option<&DbCmdHandle> {
    //     self.0.get(key)
    // }
}
impl Deref for KvsIndex {
    type Target = HashMap<String, LogMeta>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
