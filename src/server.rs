use crate::{
    engine::sled::SledKvsEngine,
    error::{KvsError, Result},
    KvStore, KvsEngine,
};
use either::Either;
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

pub struct KvsServer;
impl KvsServer {
    pub fn open(
        path: impl AsRef<Path>,
        engine: Option<KvsEngineSel>,
    ) -> Result<Either<KvStore, SledKvsEngine>> {
        let path = {
            let mut p = path.as_ref().to_path_buf();
            p.push("00engine");
            p
        };
        let engine_on_fs: Option<KvsEngineSel> = if path.is_file() {
            let mut engine = fs::File::open(&path)?;
            let mut buf = Vec::with_capacity(8);
            engine.read_to_end(&mut buf)?;
            let engine = std::str::from_utf8(&buf)?;
            Some(engine.parse()?)
        } else {
            None
        };
        let db = {
            match (engine_on_fs, engine) {
                (None, e) => {
                    let e = e.or(Some(KvsEngineSel::KvStore)).unwrap();
                    // info!(log, "creating a new db, using engine: {e}");
                    e
                }
                (Some(e), None) => {
                    // info!(log, "opening db with engine: {e}");
                    e
                }
                (Some(e), Some(e2)) if e == e2 => {
                    // info!(log, "opening db with engine: {e}");
                    e
                }
                (Some(e), Some(e2)) => {
                    let s = format!("db engine mismatch! db:`{e}`, cli: `{e2}`");
                    // error!(log, "{s}");
                    return Err(KvsError::CommandError(s).into());
                }
            }
        }
        .open(&path)?;
        Ok(db)
    }
}

#[derive(Debug, PartialEq)]
pub enum KvsEngineSel {
    KvStore,
    SledKvsEngine,
}
impl KvsEngineSel {
    pub fn open(self, path: &Path) -> crate::error::Result<Either<KvStore, SledKvsEngine>> {
        let mut f = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)?;
        write!(f, "{self}")?;
        let path = path.parent().unwrap();
        Ok(match self {
            KvsEngineSel::KvStore => Either::Left(KvStore::open(path)?),
            KvsEngineSel::SledKvsEngine => Either::Right(SledKvsEngine::open(path)?),
        })
    }
}
impl std::fmt::Display for KvsEngineSel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            KvsEngineSel::KvStore => "kvs",
            KvsEngineSel::SledKvsEngine => "sled",
        };
        write!(f, "{display}")
    }
}
impl std::str::FromStr for KvsEngineSel {
    type Err = ParseKvsEngineError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "kvs" => Ok(Self::KvStore),
            "sled" => Ok(Self::SledKvsEngine),
            s => Err(s.into()),
        }
    }
}
#[derive(Debug, Clone, failure::Fail)]
pub struct ParseKvsEngineError(String);
impl From<&str> for ParseKvsEngineError {
    fn from(s: &str) -> Self {
        Self(String::from(s))
    }
}
impl std::fmt::Display for ParseKvsEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown engine: {}", self.0)
    }
}
