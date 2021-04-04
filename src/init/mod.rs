#[allow(dead_code)]
use crate::{Error, Result};

use clap::ArgMatches;
use tera::{Context, Tera};

pub fn init(matches: &ArgMatches<'_>) -> Result<String> {
    // Fail silently?
    let shell = matches
        .value_of("shell")
        .ok_or(dev_err!("required `clap` arg absent"))?;

    let first_max_depth = matches
        .value_of("first-max-depth")
        .and_then(|depth| depth.parse::<usize>().ok());
    let next_max_depth = matches
        .value_of("next-max-depth")
        .and_then(|depth| depth.parse::<usize>().ok());

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
                include_str!("../../init/kn.fish.template"),
                &context,
                true,
            )
            .map_err(|err| dev_err!(err));

            output
        }
        _ => Err(Error::InvalidArgValue("shell".to_string())),
    }
}
