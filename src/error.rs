use std::fmt::Display;

use failure::{Error, Fail};
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Fail)]
pub struct KeyNotFoundError {
    key: String,
}
impl Display for KeyNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "key {} not found!", self.key)
    }
}

impl KeyNotFoundError {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

#[derive(Debug, Fail)]
pub struct KvsInnerError;
impl Display for KvsInnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "inner error. this is a bug")
    }
}

impl KvsInnerError {
    pub fn new() -> Self {
        Self
    }
}
