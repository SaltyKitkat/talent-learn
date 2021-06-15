use std::process::exit;

use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .before_help(env!("CARGO_PKG_DESCRIPTION"))
        .subcommands(vec![
            SubCommand::with_name("set")
                .arg(Arg::with_name("key").takes_value(true).required(true))
                .arg(Arg::with_name("value").takes_value(true).required(true)),
            SubCommand::with_name("get")
                .arg(Arg::with_name("key").takes_value(true).required(true)),
            SubCommand::with_name("remove")
                .alias("rm")
                .arg(Arg::with_name("key").takes_value(true).required(true)),
        ])
        .get_matches();
    match matches.subcommand() {
        ("set", Some(subm)) => {
            eprintln!("unimplemented");
            exit(255)
        }
        ("get", Some(subm)) => {
            eprintln!("unimplemented");
            exit(255)
        }
        ("remove", Some(subm)) => {
            eprintln!("unimplemented");
            exit(255)
        }
        _ => exit(1),
    }
}
