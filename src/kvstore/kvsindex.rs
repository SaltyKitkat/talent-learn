use super::logmisc::LogMeta;
use crate::error::{KvsError, Result};
use std::{collections::HashMap, ops::Deref};

/// A wrapper for hashmap, `insert` and `remove` functions will return the length of log
/// making calculate invalid size more convenient.
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

    /// if the key to be removed is not found, an error will be returned.
    pub(crate) fn remove(&mut self, key: &str) -> Result<usize> {
        match self.0.remove(key) {
            Some(cmd) => Ok(cmd.len()),
            None => Err(KvsError::Inner(String::from("Failed to find the key to remove")).into()),
        }
    }
}

/// make other functions for `Hashmap`, such as `get`, can be directly called.
impl Deref for KvsIndex {
    type Target = HashMap<String, LogMeta>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
