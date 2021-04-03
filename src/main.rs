#![feature(exact_size_is_empty, box_syntax)]

use std::{collections::VecDeque, io::Write, process::exit};

mod app;
mod node;
#[macro_use]
mod error;
mod utils;

use error::Error;
use utils::*;

fn main() -> Result<(), Error> {
    #[cfg(debug_assertions)]
    {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "debug");
        }
    }

    pretty_env_logger::init();

    let matches = app::app().get_matches();

    if let Some(ref matches) = matches.subcommand_matches("init") {
        let shell = matches
            .value_of("shell")
            .ok_or(dev_err!("required `clap` arg absent"))?;

        match shell {
            "fish" => print!(include_str!("../init/kn.fish")),
            _ => {}
        }

        std::io::stdout().flush()?;
    } else if let Some(ref matches) = matches.subcommand_matches("query") {
        let args = matches
            .values_of("SLICES")
            .ok_or(dev_err!("required `clap` arg absent"))?;
        let (start_dir, slices) = parse_args(args.into_iter())?;
        log::debug!("start dir: {}", start_dir.display());
        log::debug!("slices: {:?}", slices);

        if slices.is_empty() {
            print!("{}", start_dir.display());
            exit(0);
        }

        let first_level = prepare_first_level(&start_dir, slices.as_slice())
            .unwrap_or_else(|_| VecDeque::new());
        let found = find_paths(first_level);
        log::debug!("found: {:?}", found);

        if let Some(first) = found.get(0) {
            // For now just return the first path found (kinda random). What to
            // do instead?
            print!("{}", first.display());
            exit(0);
        } else {
            // Do nothing? TODO: Compare with zoxide.
            eprintln!("nothing found");
            exit(1);
        }
    }

    Ok(())
}
