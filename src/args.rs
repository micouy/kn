use std::path::PathBuf;

use crate::{Error, Result};

#[derive(Debug)]
pub enum Subcommand {
    Init { shell: Shell },
    Query { abbr: String },
    Interactive { tmpfile: PathBuf },
}

#[derive(Debug)]
pub enum Shell {
    Fish,
    Zsh,
    Bash,
}

const SUBCOMMAND_ARG: &str = "subcommand";
const SHELL_ARG: &str = "shell";

pub fn parse_args() -> Result<Subcommand> {
    let mut pargs = pico_args::Arguments::from_env();

    let subcommand = pargs
        .subcommand()?
        .ok_or(pico_args::Error::MissingArgument)?;

    match subcommand.as_str() {
        "init" => {
            let shell: String = pargs.free_from_str()?;

            let shell = match shell.as_str() {
                "fish" => Shell::Fish,
                "zsh" => Shell::Zsh,
                "bash" => Shell::Bash,
                _ => return Err(Error::InvalidArgumentValue(SHELL_ARG.to_string())),
            };

            Ok(Subcommand::Init { shell })
        }
        "query" => {
            let abbr = pargs.free_from_str()?;

            Ok(Subcommand::Query { abbr })
        }
        "interactive" => {
            let tmpfile = pargs.free_from_os_str(|arg| -> Result<_> {
                Ok(PathBuf::from(arg))
            })?;

            Ok(Subcommand::Interactive { tmpfile })
        }
        _ => Err(Error::InvalidArgumentValue(SUBCOMMAND_ARG.to_string())),
    }
}
