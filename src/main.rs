#![warn(missing_docs)]

//! Alternative to `cd`. Navigate by typing abbreviations of paths.

use std::process::exit;

#[macro_use]
pub mod utils;
pub mod abbr;
pub mod args;
pub mod error;

pub mod init;
pub mod query;

use crate::{args::Subcommand, error::Error};

/// A wrapper around the main function.
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

/// The main function.
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
