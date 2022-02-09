use failure::Fail;
use std::{error::Error, fmt::Display};
pub(crate) type KvsResult<T> = std::result::Result<T, KvsError>;
#[derive(Debug, Fail)]
pub enum KvsError {
    CommandError(String),
    CompactionError(String),
    KeyNotFound { key: String },
    Inner(String),
}
impl Display for KvsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KvsError::CommandError(s) => write!(f, "kvs-cli: {s}"),
            KvsError::CompactionError(s) => write!(f, "kvs-compact: {s}"),
            KvsError::KeyNotFound { key } => write!(f, "{key}"),
            KvsError::Inner(s) => write!(f, "kvs-inner: {s}"),
        }
    }
}
impl<E: Error> From<E> for KvsError {
    fn from(e: E) -> Self {
        Self::Inner(format!("{e}"))
    }
}
