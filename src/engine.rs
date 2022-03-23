use crate::error::KvsError;

pub mod kvstore;
pub mod sled;
pub type Result<T> = std::result::Result<T, KvsError>;

pub trait KvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn remove(&mut self, key: String) -> Result<()>;
}
