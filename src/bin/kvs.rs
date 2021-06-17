use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Config {
    #[structopt(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(StructOpt)]
enum Cmd {
    Set { key: String, value: String },
    Get { key: String },
    #[structopt(alias = "rm")] // rm is the subcmd used by test
    Remove { key: String },
}

fn main() {
    let cfg = Config::from_args();
    use Cmd::*;
    match cfg.cmd {
        Some(Set { key, value }) => {
            // we use 255 as the return code meaning that the function is unimplemented here.
            eprintln!("unimplemented");
            exit(255)
        }
        Some(Get { key }) => {
            eprintln!("unimplemented");
            exit(255)
        }
        Some(Remove { key }) => {
            eprintln!("unimplemented");
            exit(255)
        }
        _ => exit(1),
    }
}
