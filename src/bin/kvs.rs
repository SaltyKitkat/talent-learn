use kvs::KvStore;
use kvs::Result;
use std::env::current_dir;
use std::{path::PathBuf, process::exit};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Config {
    #[structopt(subcommand)]
    cmd: Option<Cmd>,
    #[structopt(parse(from_os_str))]
    db_path: Option<PathBuf>,
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
        if let Some(kvs::error::KvsError::KeyNotFound(_k)) = e.as_fail().downcast_ref() {
            println!("Key not found");
        }
        exit(1)
    }
}

fn run_app() -> Result<()> {
    let cfg = Config::from_args();
    use Cmd::*;
    let mut kvstore = KvStore::open(
        cfg.db_path.unwrap_or_else(|| current_dir().expect("failed to open current dir as working dir"))
    )
    .expect("open db file failed");
    match cfg.cmd {
        Some(Set { key, value }) => kvstore.set(key, value),
        Some(Get { key }) => {
            let value = kvstore.get(key)?;
            match value {
                Some(s) => println!("{}", s),
                None => println!("Key not found"),
            }
            Ok(())
        }
        Some(Remove { key }) => kvstore.remove(key),
        None => unreachable!(),
    }
}
