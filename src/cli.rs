use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Deserialize, Serialize, StructOpt)]
pub enum Request {
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

#[derive(Debug, Deserialize, Serialize)]
pub enum Response {
    Set(Result<(), String>),
    Get(Result<Option<String>, String>),
    Remove(Result<(), String>),
}
