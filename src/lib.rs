use std::collections::HashMap;

pub struct KvStore(HashMap<String, String>);

impl KvStore {
    pub fn new() -> Self {
        KvStore(HashMap::new())
    }

    pub fn set(&mut self, key: String, value: String) -> Option<String> {
        self.0.insert(key, value)
    }

    pub fn get(&mut self, key: String) -> Option<String> {
        self.0.get(&key).and_then(|s| Some(s.to_owned()))
    }

    pub fn remove(&mut self, key: String) -> Option<String> {
        self.0.remove(&key)
    }
}
