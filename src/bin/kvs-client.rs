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
    let r = run_app();
    if let Err(e) = r {
        if let Some(kvs::error::KvsError::KeyNotFound(k)) = e.as_fail().downcast_ref() {
            println!("Key not found");
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
            None => {
                return Err(KvsError::KeyNotFound(match cfg.request {
                    Request::Set { .. } => unreachable!(),
                    Request::Get { key } => key,
                    Request::Remove { .. } => unreachable!(),
                })
                .into())
            }
        },
        Response::Remove(_) => (),
    }
    Ok(())
}
