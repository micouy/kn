use crate::error::Error;

#[derive(Debug)]
pub enum Subcommand {
    Init { shell: Shell },
    Query { abbr: String },
}

#[derive(Debug)]
pub enum Shell {
    Fish,
    Zsh,
    Bash,
}

const SUBCOMMAND_ARG: &str = "subcommand";
const SHELL_ARG: &str = "shell";

pub fn parse_args() -> Result<Subcommand, Error> {
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
                _ => return Err(Error::InvalidArgValue(SHELL_ARG.to_string())),
            };

            Ok(Subcommand::Init { shell })
        }
        "query" => {
            let abbr = pargs.free_from_str()?;

            Ok(Subcommand::Query { abbr })
        }
        _ => Err(Error::InvalidArgValue(SUBCOMMAND_ARG.to_string())),
    }
}
