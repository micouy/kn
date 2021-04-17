#![allow(unused_parens)]

use std::process::exit;

#[macro_use]
mod utils;
mod app;
mod error;
mod init;
// TODO: Rename them i.e. find, query.
mod interactive;
mod query;

pub use error::{Error, Result};

fn main() {
    pretty_env_logger::init();

    let matches = app::app().get_matches();

    if let Some(ref matches) = matches.subcommand_matches("init") {
        match init::init(matches) {
            Ok(script) => {
                print!("{}", script);

                exit(0);
            }
            Err(error) => {
                eprintln!("{}", error);

                exit(1);
            }
        }
    } else if let Some(ref matches) = matches.subcommand_matches("query") {
        match query::query(matches) {
            Err(error) => {
                eprintln!("{}", error);

                exit(1);
            }
            Ok(()) => exit(0),
        }
    } else if let Some(ref matches) = matches.subcommand_matches("interactive")
    {
        match interactive::interactive(matches) {
            Err(error) => {
                eprintln!("{}", error);

                exit(1);
            }
            Ok(()) => exit(0),
        }
    } else {
        eprintln!("{}", dev_err!("no subcommand invoked"));

        exit(1);
    }
}
