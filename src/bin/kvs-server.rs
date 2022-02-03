use either::Either;
use failure::Fail;
use kvs::{
    cli::{Request, Response},
    engine::sled::SledKvsEngine,
    error::KvsError,
    KvStore, KvsEngine,
};
use slog::{error, info, o, Drain};
use std::{
    env::current_dir,
    fs,
    io::{BufRead, BufReader, BufWriter, Read},
    net::{SocketAddr, TcpListener},
    str::FromStr,
};
use structopt::{clap::crate_version, StructOpt};

#[derive(Debug, PartialEq)]
enum KvsEngineSel {
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
impl FromStr for KvsEngineSel {
    type Err = ParseKvsEngineError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // let s = s.to_ascii_lowercase();
        // match s.as_str() {
        match s {
            "kvs" /* | "kvstore" */ => Ok(Self::KvStore),
            "sled" => Ok(Self::SledKvsEngine),
            s => Err(s.into()),
        }
    }
}

#[derive(Debug, Clone, Fail)]
struct ParseKvsEngineError(String);
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

#[derive(StructOpt)]
struct Config {
    #[structopt(long, global = true, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
    #[structopt(long, global = true)]
    engine: Option<KvsEngineSel>,
}

fn run_app() -> kvs::error::Result<()> {
    let cfg = Config::from_args();
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());
    info!(log, "kvs-server started!");
    info!(log, "version: {}", crate_version!());
    let path = {
        let mut path = current_dir()?;
        info!(log, "Opening db in path: {}", path.to_string_lossy());
        path.push("00engine");
        path
    };
    // read engine from path
    let engine_on_fs: Option<KvsEngineSel> = if path.is_file() {
        let mut engine = fs::File::open(&path)?;
        let mut buf = Vec::with_capacity(8);
        engine.read_to_end(&mut buf)?;
        let engine = std::str::from_utf8(&buf)?;
        info!(log, "got engine `{engine}` from fs");
        Some(engine.parse()?)
    } else {
        None
    };

    // if new: read engine from cli, create new db
    // else: if engine from cli is different: error and exit
    // else: opendb
    let engine = match (engine_on_fs, cfg.engine) {
        (None, e) => {
            let e = e.or(Some(KvsEngineSel::KvStore)).unwrap();
            info!(log, "creating a new db, using engine: {e}");
            e
        }
        (Some(e), None) => {
            info!(log, "opening db with engine: {e}");
            e
        }
        (Some(e), Some(e2)) if e == e2 => {
            info!(log, "opening db with engine: {e}");
            e
        }
        (Some(e), Some(e2)) => {
            let s = format!("db engine mismatch! db:`{e}`, cli: `{e2}`");
            error!(log, "{s}");
            return Err(KvsError::CommandError(s).into());
        }
    };

    // open db
    let mut db = match engine {
        KvsEngineSel::KvStore => Either::Left(KvStore::open(path.parent().unwrap())?),
        KvsEngineSel::SledKvsEngine => Either::Right(SledKvsEngine::open(path.parent().unwrap())?),
    };
    // setup listener
    let listener = TcpListener::bind(cfg.addr)?;
    info!(log, "Listening on socket: {}", cfg.addr);
    for stream in listener.incoming() {
        let stream = stream?;
        let mut buf = String::new();
        let mut reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);
        loop {
            buf.clear();
            let size = reader.read_line(&mut buf)?;
            if size == 0 {
                break;
            }
            // parse and execute
            let cmd: Request = serde_json::de::from_str(&buf)?;
            let response = match cmd {
                Request::Set { key, value } => {
                    db.set(key, value)?; // todo: log
                    Response::Set
                }
                Request::Get { key } => Response::Get(db.get(key)?),
                Request::Remove { key } => {
                    db.remove(key)?; // todo: log
                    Response::Rm
                }
            };
            // return result
            serde_json::ser::to_writer(&mut writer, &response)?;
        }
    }
    Ok(())
}
fn main() {
    match run_app() {
        Ok(_) => todo!(),
        Err(_) => todo!(),
    }
}
