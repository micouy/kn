#![feature(exact_size_is_empty, box_syntax)]
#![allow(unused_parens)]

use std::{collections::VecDeque, process::exit};

use tera::{Context, Tera};

mod app;
mod node;
#[macro_use]
mod error;
mod utils;

use error::Error;
use utils::*;

fn main() {
    match inner() {
        Err(error) => println!("{}", error),
        Ok(()) => {}
    }
}

fn inner() -> Result<(), Error> {
    pretty_env_logger::init();

    let matches = app::app().get_matches();

    if let Some(ref matches) = matches.subcommand_matches("init") {
        let shell = matches
            .value_of("shell")
            .ok_or(dev_err!("required `clap` arg absent"))?;

        let first_max_depth = matches
            .value_of("first-max-depth")
            .and_then(|depth| depth.parse::<u32>().ok());
        let next_max_depth = matches
            .value_of("next-max-depth")
            .and_then(|depth| depth.parse::<u32>().ok());
        let mut context = Context::new();
        if let Some(depth) = first_max_depth {
            context.insert("first_max_depth", &depth);
        }
        if let Some(depth) = next_max_depth {
            context.insert("next_max_depth", &depth);
        }

        match shell {
            "fish" => {
                let output = Tera::one_off(
                    include_str!("../init/kn.fish.template"),
                    &context,
                    true,
                )
                .map_err(|err| dev_err!(err))?;

                print!("{}", output);
                exit(0);
            }
            _ => {}
        }
    } else if let Some(ref matches) = matches.subcommand_matches("query") {
        let args = matches
            .values_of("SLICES")
            .ok_or(dev_err!("required `clap` arg absent"))?;

        let first_max_depth = matches
            .value_of("first-max-depth")
            .map(|depth| depth.parse::<u32>())
            .transpose()
            .map_err(|_| {
                Error::InvalidArgValue("first-max-depth".to_string())
            })?;

        let next_max_depth = matches
            .value_of("next-max-depth")
            .map(|depth| depth.parse::<u32>())
            .transpose()
            .map_err(|_| {
                Error::InvalidArgValue("next-max-depth".to_string())
            })?;

        let (start_dir, slices) = parse_args(args.into_iter())?;
        log::debug!("start dir: {}", start_dir.display());
        log::debug!("slices: {:?}", slices);

        if slices.is_empty() {
            print!("{}", start_dir.display());
            exit(0);
        }

        let first_level = prepare_first_level(&start_dir, slices.as_slice())
            .unwrap_or_else(|_| VecDeque::new());
        let found = find_paths(first_level, first_max_depth, next_max_depth);
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
