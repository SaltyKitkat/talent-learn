use kvs::{
    engine::sled::SledKvsEngine,
    error::KvsError,
    server::{KvsEngineSel, KvsServer},
    KvStore, Result,
};
use slog::{error, info, o, Drain, Logger};
use std::{env::current_dir, fs, io, net::SocketAddr, path::Path, process::exit};
use structopt::{clap::crate_version, StructOpt};

#[derive(StructOpt)]
struct Config {
    #[structopt(long, global = true, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
    #[structopt(long, global = true)]
    engine: Option<KvsEngineSel>,
}

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());
    if let Err(e) = run_app(&logger) {
        error!(logger, "{e}");
        exit(1);
    }
}

fn run_app(log: &Logger) -> Result<()> {
    let cfg = Config::from_args();
    info!(log, "kvs-server started!");
    info!(log, "version: {}", crate_version!());
    let path = current_dir()?;
    let engine: KvsEngineSel = {
        let path = path.join("00engine");
        let e_cli = cfg.engine;
        match current_engine(&path)? {
            Some(e_disk) => {
                if let Some(e_cli) = e_cli {
                    if e_cli != e_disk {
                        let e = KvsError::MisMatchEngine { e_disk, e_cli };
                        error!(log, "{e}");
                        return Err(e);
                    }
                }
                e_disk
            }
            None => {
                let engine = e_cli.unwrap_or_default();
                fs::write(&path, engine.to_string())?;
                engine
            }
        }
    };
    info!(log, "using storage engine: {engine}");
    let server = match engine {
        KvsEngineSel::KvStore => KvsServer::new(KvStore::open(&path)?, log),
        KvsEngineSel::SledKvsEngine => KvsServer::new(SledKvsEngine::open(&path)?, log),
    };
    info!(log, "server listening on socket: {}", cfg.addr);
    server.run(cfg.addr)
}

fn current_engine(path: &Path) -> Result<Option<KvsEngineSel>> {
    match fs::read_to_string(path) {
        Ok(buf) => Ok(Some(buf.parse()?)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
