pub mod kvstore;
pub mod sled;
use failure::Error;

pub trait KvsEngine {
    // type Result<T> = Result<T, Self::Error>;
    fn set(&mut self, key: String, value: String) -> Result<(), Error>;
    fn get(&mut self, key: String) -> Result<Option<String>, Error>;
    fn remove(&mut self, key: String) -> Result<(), Error>;
}
