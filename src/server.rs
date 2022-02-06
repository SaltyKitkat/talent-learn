use crate::{
    cli::{Request, Response},
    engine::sled::SledKvsEngine,
    error::{KvsError, Result},
    KvStore, KvsEngine,
};
use std::{
    fmt::Display,
    fs,
    io::{self, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    path::Path,
};

pub struct KvsServer(Box<dyn KvsEngine>);
impl KvsServer {
    pub fn new(e: impl KvsEngine + 'static) -> Self {
        Self(Box::new(e))
    }
    pub fn run(mut self, socket: impl ToSocketAddrs) -> Result<()> {
        let listener = TcpListener::bind(socket)?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = self.serve(stream) {
                        todo!()
                    }
                }
                Err(e) => todo!(),
            }
        }
        todo!()
    }

    // we can use `?` to throw errors, which will be handled by upper function
    fn serve(&mut self, stream: TcpStream) -> Result<()> {
        let mut buf = String::new();
        let reader = io::BufReader::new(&stream);
        let mut writer = io::BufWriter::new(&stream);
        let req_reader = serde_json::Deserializer::from_reader(reader).into_iter::<Request>();
        #[inline]
        fn trans_err<T, E: Display>(
            result: std::result::Result<T, E>,
        ) -> std::result::Result<T, String> {
            result.map_err(|e| e.to_string())
        }
        for req in req_reader {
            let response = match req? {
                Request::Set { key, value } => Response::Set(trans_err(self.0.set(key, value))),
                Request::Get { key } => Response::Get(trans_err(self.0.get(key))),
                Request::Remove { key } => Response::Remove(trans_err(self.0.remove(key))),
            };
            serde_json::to_writer(&mut writer, &response)?;
            writer.flush()?
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum KvsEngineSel {
    KvStore,
    SledKvsEngine,
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
