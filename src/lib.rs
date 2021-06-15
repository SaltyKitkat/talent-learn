#![deny(missing_docs)]

//! this is a crate doc
use std::collections::HashMap;

/// there is just a warpper for a HashMap.
pub struct KvStore(HashMap<String, String>);

impl KvStore {
    /// As its name.
    pub fn new() -> Self {
        KvStore(HashMap::new())
    }
    /// add a (key, value) pair into the KvStore
    pub fn set(&mut self, key: String, value: String) -> Option<String> {
        self.0.insert(key, value)
    }
    /// get the value due to the key
    pub fn get(&mut self, key: String) -> Option<String> {
        self.0.get(&key).and_then(|s| Some(s.to_owned()))
    }
    /// remove one entry due to the key
    pub fn remove(&mut self, key: String) -> Option<String> {
        self.0.remove(&key)
    }
}
