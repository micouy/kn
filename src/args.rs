//! Arg parsing.

use std::path::PathBuf;

use crate::error::Error;

/// Subcommand with its args.
#[derive(Debug)]
pub enum Subcommand {
    /// The [`init`](crate::init::init) subcommand.
    Init {
        /// User's shell.
        shell: Shell,

        /// The value of the `--exclude-old-pwd` flag.
        exclude_old_pwd: bool,
    },
    /// The [`query`](crate::query::query) subcommand.
    Query {
        /// The abbr.
        abbr: String,

        /// Path excluded from search.
        excluded: Option<PathBuf>,
    },
}

/// The value of the `--shell` arg.
#[derive(Debug)]
pub enum Shell {
    #[allow(missing_docs)]
    Fish,

    #[allow(missing_docs)]
    Zsh,

    #[allow(missing_docs)]
    Bash,
}

const SUBCOMMAND_ARG: &str = "subcommand";
const SHELL_ARG: &str = "--shell";
const ABBR_ARG: &str = "--abbr";
const EXCLUDE_OLD_PWD_ARG: &str = "--exclude-old-pwd";
const EXCLUDE_ARG: &str = "--exclude";
const FISH_ARG: &str = "fish";
const BASH_ARG: &str = "bash";
const ZSH_ARG: &str = "zsh";
const INIT_SUBCOMMAND: &str = "init";
const QUERY_SUBCOMMAND: &str = "query";

/// Parses CLI args.
pub fn parse_args() -> Result<Subcommand, Error> {
    let mut pargs = pico_args::Arguments::from_env();

    let subcommand = pargs
        .subcommand()?
        .ok_or(pico_args::Error::MissingArgument)?;

    match subcommand.as_str() {
        INIT_SUBCOMMAND => {
            let shell: String = pargs.value_from_str(SHELL_ARG)?;

            let shell = match shell.as_str() {
                FISH_ARG => Shell::Fish,
                ZSH_ARG => Shell::Zsh,
                BASH_ARG => Shell::Bash,
                _ => return Err(Error::InvalidArgValue(SHELL_ARG.to_string())),
            };

            let exclude_old_pwd = pargs.contains(EXCLUDE_OLD_PWD_ARG);

            Ok(Subcommand::Init {
                shell,
                exclude_old_pwd,
            })
        }
        QUERY_SUBCOMMAND => {
            let abbr = pargs.value_from_str(ABBR_ARG)?;
            let excluded = pargs
                .opt_value_from_os_str::<_, _, !>(EXCLUDE_ARG, |os_str| {
                    Ok(PathBuf::from(os_str))
                })?;

            Ok(Subcommand::Query { abbr, excluded })
        }
        _ => Err(Error::InvalidArgValue(SUBCOMMAND_ARG.to_string())),
    }
}
