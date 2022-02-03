use kvs::{
    cli::{Request, Response},
    error::KvsError,
    server::{KvsEngineSel, KvsServer},
    KvsEngine,
};
use slog::{error, info, o, Drain};
use std::{
    env::current_dir,
    fs,
    io::{BufRead, BufReader, BufWriter, Read},
    net::{SocketAddr, TcpListener},
};
use structopt::{clap::crate_version, StructOpt};

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
    let mut db = KvsServer::open(current_dir()?, cfg.engine)?;

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
