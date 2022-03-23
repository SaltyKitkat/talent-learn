use kvs::{
    cli::{Request, Response},
    client::KvsClient,
    error::KvsError,
    Result,
};
use std::{net::SocketAddr, process::exit};
use structopt::StructOpt;
#[derive(StructOpt)]
struct Config {
    #[structopt(subcommand)]
    request: Request,
    #[structopt(long, global = true, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
}

fn main() {
    if let Err(e) = run_app() {
        if let Some(KvsError::KeyNotFound { key: _ }) = e.as_fail().downcast_ref() {
            eprintln!("Key not found");
        }
        exit(1)
    }
}

fn run_app() -> Result<()> {
    let cfg = Config::from_args();
    let mut client = KvsClient::connect(cfg.addr)?;
    match client.send_request(&cfg.request)? {
        Response::Set(_) => (),
        Response::Get(r) => match r.map_err(|s| KvsError::Inner(s))? {
            Some(value) => println!("{value}"),
            None => println!("Key not found"),
        },
        Response::Remove(result) => {
            return result.map_err(|e| KvsError::KeyNotFound { key: e }.into())
        }
    }
    Ok(())
}
