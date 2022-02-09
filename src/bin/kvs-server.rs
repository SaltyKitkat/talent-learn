use kvs::{
    engine::sled::SledKvsEngine,
    error::KvsError,
    server::{KvsEngineSel, KvsServer},
    KvStore, Result,
};
use slog::{error, info, o, Drain};
use std::{env::current_dir, fs, io, net::SocketAddr};
use structopt::{clap::crate_version, StructOpt};

const DEFAULT_KVSENGINE: KvsEngineSel = KvsEngineSel::KvStore;
#[derive(StructOpt)]
struct Config {
    #[structopt(long, global = true, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
    #[structopt(long, global = true)]
    engine: Option<KvsEngineSel>,
}

fn run_app() -> Result<()> {
    let cfg = Config::from_args();
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());
    info!(log, "kvs-server started!");
    info!(log, "version: {}", crate_version!());
    let path = current_dir()?;
    let engine: KvsEngineSel = {
        let path = path.join("00engine");
        match fs::read(&path) {
            Ok(v) => {
                let s = std::str::from_utf8(&v)?;
                let e1 = s.parse()?;
                match cfg.engine {
                    Some(e2) if e1 != e2 => {
                        return Err(KvsError::CommandError("engine mismatch".into()).into())
                    }
                    _ => e1,
                }
            }
            Err(e) if matches!(e.kind(), io::ErrorKind::NotFound) => {
                let engine = cfg.engine.or(Some(DEFAULT_KVSENGINE)).unwrap();
                fs::write(&path, engine.to_string())?;
                engine
            }
            Err(e) => return Err(e.into()),
        }
    };
    info!(log, "using storage engine: {engine}");
    let server = match engine {
        KvsEngineSel::KvStore => KvsServer::new(KvStore::open(&path)?),
        KvsEngineSel::SledKvsEngine => KvsServer::new(SledKvsEngine::open(&path)?),
    };
    info!(log, "server listening on socket: {}", cfg.addr);
    server.run(cfg.addr)
}
fn main() {
    run_app().unwrap(); // todo: handle error
}
