use crate::KvStore;
use failure::Error;

pub trait KvsEngine {
    // type Result<T> = Result<T, Self::Error>;
    fn set(&mut self, key: String, value: String) -> Result<(), Error>;
    fn get(&mut self, key: String) -> Result<Option<String>, Error>;
    fn remove(&mut self, key: String) -> Result<(), Error>;
}

impl KvsEngine for KvStore {
    fn set(&mut self, key: String, value: String) -> Result<(), Error> {
        KvStore::set(self, key, value)
    }

    fn get(&mut self, key: String) -> Result<Option<String>, Error> {
        KvStore::get(self, key)
    }

    fn remove(&mut self, key: String) -> Result<(), Error> {
        KvStore::remove(self, key)
    }
}
