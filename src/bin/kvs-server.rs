use slog::{error, info, o, Drain};
use std::{
    env::current_dir,
    io::{BufRead, BufReader, BufWriter},
    net::{SocketAddr, TcpListener},
};
use structopt::{clap::crate_version, StructOpt};
#[derive(StructOpt)]
struct Config {
    #[structopt(long, global = true, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
    #[structopt(long, global = true, default_value = "kvs")]
    engine: String,
}

use kvs::{error::KvsError, KvStore, Result};
fn run_app() -> Result<()> {
    let cfg = Config::from_args();
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());
    info!(log, "kvs-server started!");
    info!(log, "version: {}", crate_version!());
    let path = current_dir()?;
    info!(log, "Opening db in path: {}", path.to_string_lossy());
    let engine = match cfg.engine.as_str() {
        "kvs" => KvStore::open(path)?,
        "sled" => todo!(),
        _ => {
            error!(log, "unknown engine from cli: {}", cfg.engine);
            Err(KvsError::CommandError(format!(
                "unknown engine: {}",
                cfg.engine
            )))?
        }
    };
    // read engine from path
    // if new: read engine from cli, create new db
    // else: log: warning: musing engine .., .. from cli is ignored
    // open db
    // setup listener
    let listener = TcpListener::bind(cfg.addr)?;
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
            let cmd = todo!("parse the cmd");
            todo!("exec the cmd");
            // return result
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
