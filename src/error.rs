use failure::{Error, Fail};
use std::fmt::Display;
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Fail)]
pub enum KvsError {
    CommandError(String),
    KeyNotFound(String),
    Inner(String),
}
impl Display for KvsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "some error occurred.")
    }
}
