use std::{
    io::{BufRead, BufReader, BufWriter},
    net::{SocketAddr, TcpListener},
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Config {
    #[structopt(long, global = true, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
    #[structopt(long, global = true)]
    engine: Option<String>,
}

use kvs::Result;
fn run_app() -> Result<()> {
    let cfg = Config::from_args();
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
