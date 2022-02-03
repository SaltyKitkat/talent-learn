use kvs::error::KvsError;
use kvs::KvStore;
use kvs::KvsEngine;
use kvs::Result;
use std::{path::PathBuf, process::exit};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Config {
    #[structopt(subcommand)]
    cmd: Option<Cmd>,
    #[structopt(default_value = ".")]
    db_path: PathBuf,
}

#[derive(StructOpt)]
enum Cmd {
    Set {
        key: String,
        value: String,
    },
    Get {
        key: String,
    },
    #[structopt(alias = "rm")] // rm is the subcmd used by test
    Remove {
        key: String,
    },
}

fn main() {
    let r = run_app();
    if let Err(e) = r {
        if let Some(KvsError::KeyNotFound(_k)) = e.as_fail().downcast_ref() {
            println!("Key not found");
        }
        exit(1)
    }
}

fn run_app() -> Result<()> {
    let cfg = Config::from_args();
    if let Some(cmd) = cfg.cmd {
        use Cmd::*;
        let mut kvstore = KvStore::open(cfg.db_path).expect("open db file failed");
        match cmd {
            Set { key, value } => kvstore.set(key, value),
            Get { key } => {
                let value = kvstore.get(key)?;
                match value {
                    Some(s) => println!("{s}"),
                    None => println!("Key not found"),
                }
                Ok(())
            }
            Remove { key } => kvstore.remove(key),
        }
    } else {
        eprintln!("run `kvs --help` to get help messages");
        Err(KvsError::CommandError(String::from("unknown command")).into())
    }
}
