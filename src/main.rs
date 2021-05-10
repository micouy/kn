#![allow(unused_parens)]
#![feature(destructuring_assignment)]

use std::{fs, process::exit};

#[macro_use]
mod utils;
mod app;
mod error;
mod search;

mod init;
mod interactive;
mod query;

pub use error::{Error, Result};

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
    let matches = app::app().get_matches();

    if let Some(ref matches) = matches.subcommand_matches("init") {
        match init::init(matches) {
            Ok(script) => {
                print!("{}", script);

                Ok(())
            }
            Err(error) => Err(error),
        }
    } else if let Some(ref matches) = matches.subcommand_matches("query") {
        // TODO: Write the result to file?
        let abbr = matches
            .value_of_os("ABBR")
            .ok_or(dev_err!("required `clap` arg absent"))?;
        match query::query(abbr) {
            Err(error) => Err(error),
            Ok(path) => {
                println!("{}", path.display());

                Ok(())
            }
        }
    } else if let Some(ref matches) = matches.subcommand_matches("interactive")
    {
        let file = matches
            .value_of_os("TMP_FILE")
            .ok_or(dev_err!("required arg absent"))?;

        let found_path = interactive::interactive()?;
        let found_path =
            found_path.to_str().ok_or(dev_err!("invalid Unicode"))?;
        fs::write(file, found_path)?;

        Ok(())
    } else {
        Err(dev_err!("no subcommand invoked"))
    }
}
