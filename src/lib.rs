pub use error::Result;
pub use kvstore::KvStore;

pub mod error;
mod kvstore;


use structopt::StructOpt;
#[derive(StructOpt)]
pub enum Cmd {
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

