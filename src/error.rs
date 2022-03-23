use failure::Fail;
use std::{error::Error, fmt::Display};

use crate::server::KvsEngineSel;
pub(crate) type KvsResult<T> = std::result::Result<T, KvsError>;
#[derive(Debug, Fail)]
pub enum KvsError {
    CommandError(&'static str),
    CompactionError(String),
    Inner(String),
    InvalidEngine(String),
    MisMatchEngine {
        e_disk: KvsEngineSel,
        e_cli: KvsEngineSel,
    },
    KeyNotFound {
        key: String,
    },
}
impl Display for KvsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KvsError::CommandError(s) => write!(f, "kvs-cli: {s}"),
            KvsError::CompactionError(s) => write!(f, "kvs-compact: {s}"),
            KvsError::Inner(s) => write!(f, "kvs-inner: {s}"),
            KvsError::InvalidEngine(s) => write!(
                f,
                "kvs: invalid engine `{s}`, choose either `kvs` or `sled`"
            ),
            KvsError::MisMatchEngine { e_disk, e_cli } => write!(
                f,
                "engine from cli `{e_cli}` is different from engine on disk `{e_disk}`"
            ),
            KvsError::KeyNotFound { key } => write!(f, "{key}"),
        }
    }
}
impl<E: Error> From<E> for KvsError {
    fn from(e: E) -> Self {
        Self::Inner(format!("{e}"))
    }
}
