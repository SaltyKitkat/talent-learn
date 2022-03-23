use std::io;

use crate::server::KvsEngineSel;
use thiserror::Error;

pub(crate) type KvsResult<T> = std::result::Result<T, KvsError>;
#[derive(Debug, Error)]
pub enum KvsError {
    #[error("kvs-cli: {0}")]
    CommandError(&'static str),

    #[error("kvs-compact: {0}")]
    CompactionError(String),

    #[error("kvs-inner: {0}")]
    Inner(String),

    #[error("kvs: invalid engine `{0}`, choose either `kvs` or `sled`")]
    InvalidEngine(String),
    #[error("kvs-io: {source}")]
    IO {
        #[from]
        source: io::Error,
    },

    #[error("engine from cli `{e_cli}` is different from engine on disk `{e_disk}")]
    MisMatchEngine {
        e_disk: KvsEngineSel,
        e_cli: KvsEngineSel,
    },

    #[error("{key}")]
    KeyNotFound { key: String },

    #[error("serde: {source}")]
    Serde {
        #[from]
        source: serde_json::Error,
    },

    #[error("sled: {source}")]
    Sled {
        #[from]
        source: sled::Error,
    },
}
