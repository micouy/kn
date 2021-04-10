#![feature(exact_size_is_empty, box_syntax)]
#![allow(unused_parens)]

use std::process::exit;


#[macro_use]
mod utils;
mod app;
mod error;
mod init;
mod query;


pub use error::Error;


pub type Result<T> = std::result::Result<T, Error>;


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
            Ok(found) =>
            // TODO: Order the findings?
                if let Some(first) = found.get(0) {
                    print!("{}", first.display());


                    exit(0);
                } else {
                    eprintln!("nothing found");


                    exit(1);
                },
        }
    }

    // TODO: Display error if no subcommand was invoked?
}
