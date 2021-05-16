#![allow(unused_parens)]

use std::{fs, process::exit};

#[macro_use]
mod utils;
// mod app;
mod args;
mod error;
mod search;

mod init;
mod interactive;
mod query;

pub use error::{Error, Result};

use args::{parse_args, Subcommand};

fn main() {
    match _main() {
        Err(error) => {
            eprintln!("{}", error);
            exit(1);
        }
        Ok(()) => exit(0),
    }
}

fn _main() -> Result<()> {
    let subcommand = parse_args()?;

    match subcommand {
        Subcommand::Init { shell } => {
            let script = init::init(shell);
            print!("{}", script);

            Ok(())
        }
        Subcommand::Query { abbr } => match query::query(abbr) {
            Err(error) => Err(error),
            Ok(path) => {
                println!("{}", path.display());

                Ok(())
            }
        },
        Subcommand::Interactive { tmpfile } => {
            let found_path = interactive::interactive()?;
            let found_path =
                found_path.to_str().ok_or(dev_err!("invalid Unicode"))?;
            fs::write(tmpfile, found_path)?;

            Ok(())
        }
    }
}
