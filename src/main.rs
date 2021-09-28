#![warn(missing_docs)]
#![feature(pattern)]
#![feature(never_type)]

//! Alternative to `cd`. Navigate by typing abbreviations of paths.

use std::process::exit;

#[macro_use]
mod utils;
mod abbr;
mod args;
mod error;

mod init;
mod query;

use crate::{args::Subcommand, error::Error};

#[allow(missing_docs)]
fn main() {
    match _main() {
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
        Ok(()) => {
            exit(0);
        }
    }
}

fn _main() -> Result<(), Error> {
    let subcommand = args::parse_args()?;

    match subcommand {
        Subcommand::Init {
            shell,
            exclude_old_pwd,
        } => {
            let script = init::init(shell, exclude_old_pwd);
            print!("{}", script);

            Ok(())
        }
        Subcommand::Query { abbr, excluded } => {
            match query::query(&abbr, excluded) {
                Err(error) => Err(error),
                Ok(path) => {
                    println!("{}", path.display());

                    Ok(())
                }
            }
        }
    }
}
