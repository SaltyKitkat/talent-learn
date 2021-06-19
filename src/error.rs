use std::fmt::Display;

use failure::{Error, Fail};
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Fail)]
pub enum KvsError {
    KeyNotFound(String),
    Inner,
    IO,
}
impl Display for KvsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "some error occurred.")
    }
}