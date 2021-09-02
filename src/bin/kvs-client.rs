use kvs::Cmd;
use kvs::KvStore;
use kvs::Result;
use slog::info;
use std::env::current_dir;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::{path::PathBuf, process::exit};
use structopt::StructOpt;
#[derive(StructOpt)]
struct Config {
    #[structopt(subcommand)]
    cmd: Cmd,
    #[structopt(short = "p", long, global = true)]
    db_path: Option<PathBuf>,
    #[structopt(long, global = true, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
}

fn main() {
    let r = run_app();
    if let Err(e) = r {
        if let Some(kvs::error::KvsError::KeyNotFound(_k)) = e.as_fail().downcast_ref() {
            println!("Key not found");
        }
        exit(1)
    }
}

fn run_app() -> Result<()> {
    let cfg = Config::from_args();
    let socket = cfg.addr;
    let stream = TcpStream::connect(socket)?;
    Ok(())
}
