#![feature(exact_size_is_empty, box_syntax)]
#![allow(unused_parens)]

use std::process::exit;

#[macro_use]
mod error;
mod app;
mod init;
mod query;

pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

fn main() {
    match inner() {
        Err(error) => println!("{}", error),
        Ok(()) => {}
    }
}

fn inner() -> Result<()> {
    pretty_env_logger::init();

    let matches = app::app().get_matches();

    if let Some(ref matches) = matches.subcommand_matches("init") {
        let script = init::init(matches)?;
        print!("{}", script);

        exit(0);
    } else if let Some(ref matches) = matches.subcommand_matches("query") {
        let found = query::query(matches)?;

        if let Some(first) = found.get(0) {
            print!("{}", first.display());

            exit(0);
        } else {
            eprintln!("nothing found");

            exit(1);
        }
    }

    Ok(())
}
