use crate::{
    cli::{Request, Response},
    error::KvsError,
    KvsEngine, Result,
};
use slog::{warn, Logger};
use std::{
    fmt::Display,
    io::{self, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

pub struct KvsServer<'log> {
    engine: Box<dyn KvsEngine>,
    logger: &'log Logger,
}
impl<'log> KvsServer<'log> {
    pub fn new(engine: impl KvsEngine + 'static, logger: &'log Logger) -> Self {
        Self {
            engine: Box::new(engine),
            logger,
        }
    }
    pub fn run(mut self, socket: impl ToSocketAddrs) -> Result<()> {
        let listener = TcpListener::bind(socket)?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = self.serve(stream) {
                        warn!(self.logger, "{e}");
                    }
                }
                Err(e) => warn!(self.logger, "{e}"),
            }
        }
        unreachable!()
    }

    // we can use `?` to throw errors, which will be handled by upper function `run`
    fn serve(&mut self, stream: TcpStream) -> Result<()> {
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
                Request::Set { key, value } => {
                    Response::Set(trans_err(self.engine.set(key, value)))
                }
                Request::Get { key } => Response::Get(trans_err(self.engine.get(key))),
                Request::Remove { key } => Response::Remove(trans_err(self.engine.remove(key))),
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
    type Err = KvsError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "kvs" => Ok(Self::KvStore),
            "sled" => Ok(Self::SledKvsEngine),
            s => Err(KvsError::InvalidEngine(s.to_string())),
        }
    }
}
