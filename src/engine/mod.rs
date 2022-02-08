pub mod kvstore;
pub mod sled;
use failure::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub trait KvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn remove(&mut self, key: String) -> Result<()>;
}
