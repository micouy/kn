#![feature(exact_size_is_empty)]

use clap::{App, AppSettings, Arg, SubCommand};
use regex::Regex;
use thiserror::Error;

use std::{
    convert::AsRef,
    fs::{read_dir, DirEntry},
    io::Write,
    path::{Path, PathBuf},
    process::exit,
};


/// A container for an entry and args left to match.
pub struct EntryNode<'a>(pub PathBuf, pub &'a [Regex]);

/// `kn` error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Internal error at {file}:{line}. Cause: {cause}. If you see this, contact the dev.")]
    DevError {
        line: u32,
        file: &'static str,
        cause: &'static str,
    },
    #[error(
        "Invalid slice. Slices should only contain alphanumeric characters."
    )]
    InvalidSlice,
    #[error("{0}")]
    IO(#[from] std::io::Error),
}

macro_rules! dev_err {
    ($cause:expr) => {
        Error::DevError {
            line: line!(),
            file: file!(),
            cause: $cause,
        }
    };
}

fn main() -> Result<(), Error> {
    let matches = App::new(env!("CARGO_BIN_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        // Add dots at the end of messages.
        .help_message("Prints help information.")
        .version_message("Prints version information.")
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("init")
                .help("Get init script for your shell.")
                .arg(
                    Arg::with_name("shell")
                        .possible_values(&["fish"])
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("query")
                .setting(AppSettings::TrailingVarArg)
                .help("Query directory matching given slices. If the first slice is a valid dir path, the search begins there.")
                .arg(
                    Arg::with_name("SLICES")
                        .help("Slices of path to be matched.")
                        .index(1)
                        .multiple(true)
                        .required_unless("start-dir"),
                ),
        )
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("init") {
        let shell = matches
            .value_of("shell")
            .ok_or(dev_err!("absent required `clap` arg"))?;

        match shell {
            "fish" => print!(include_str!("../init/kn.fish")),
            _ => {}
        }

        std::io::stdout().flush()?;
    } else if let Some(ref matches) = matches.subcommand_matches("query") {
        let args = matches
            .values_of("SLICES")
            .ok_or(dev_err!("absent required `clap` arg"))?;

        let (start_dir, slices) = parse_args(args.into_iter())?;

        if slices.is_empty() {
            print!("{}", start_dir.display());
            exit(0);
        }

        let first_level =
            prepare_first_level(&start_dir, &slices).unwrap_or_else(Vec::new);
        let found = find_paths(first_level);

        if let Some(first) = found.get(0) {
            // For now just return one path (kinda random). What to do instead?
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

/// Find paths matching slices, beginning search at `start_dir`.
pub fn find_paths<'a>(first_level: Vec<EntryNode<'a>>) -> Vec<PathBuf> {
    let mut levels: Vec<Vec<EntryNode>> = vec![first_level];
    let mut found: Vec<PathBuf> = vec![];

    'search: loop {
        let entries = levels.last().unwrap();
        let mut new_level = vec![];

        if entries.is_empty() || !found.is_empty() {
            // Either nothing left to search or no need to search.
            break 'search;
        }

        for entry in entries {
            handle_entry(entry, &mut found, &mut new_level);
        }

        levels.push(new_level);
    }

    return found;
}

/// Generate regular expressions from provided args.
pub fn parse_args<'a, A>(args: A) -> Result<(PathBuf, Vec<Regex>), Error>
where
    A: Iterator<Item = &'a str>,
{
    let mut args = args.peekable();
    if args.peek().is_none() {
        return Err(dev_err!("empty args"));
    }

    // If the first arg is a valid dir path, remove it from
    // the list of slices begin the search there.
    let start_dir = args
        .next_if(|first| AsRef::<Path>::as_ref(first).is_dir())
        .map(|start_dir| Ok(PathBuf::from(start_dir)))
        .unwrap_or_else(std::env::current_dir)?;

    // Check if the slices contain alphanumeric characters.
    let validator = Regex::new("^[[:alnum:]]+$").unwrap();
    let arg_patterns = args
        .map(|arg| {
            if validator.is_match(&arg) {
                Ok(format!("^.*{}.*$", arg))
            } else {
                Err(Error::InvalidSlice)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let args = arg_patterns
        .iter()
        .map(|arg_pattern| Regex::new(arg_pattern))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| dev_err!("regex creation"))?;

    Ok((start_dir, args))
}

/// Check if the last component of the path matches the regular expression.
pub fn matches<P>(path: P, re: &Regex) -> bool
where
    P: AsRef<Path>,
{
    path.as_ref()
        .file_name()
        .map(|os_str| {
            let lossy = os_str.to_string_lossy();

            re.is_match(&lossy)
        })
        .unwrap_or(false)
}

/// Generate the first level of [`EntryNode`](crate::EntryNode)s.
pub fn prepare_first_level<'a, P>(
    path: P,
    args: &'a [Regex],
) -> Option<Vec<EntryNode<'a>>>
where
    P: AsRef<Path>,
{
    let entries: Vec<DirEntry> = read_dir(path)
        .unwrap()
        .filter_map(|res| res.ok()) // Ignore entires that cannot be accessed.
        .filter(|entry| {
            entry.file_type().map(|meta| meta.is_dir()).unwrap_or(false)
        })
        .collect();
    let arg = args.get(0)?;
    let mut level = vec![];

    for entry in entries {
        let path = entry.path();

        if matches(&path, arg) {
            level.push(EntryNode(path, &args[1..]));
        } else {
            level.push(EntryNode(path, &args[..]));
        }
    }

    Some(level)
}

/// Either add entry to found entries or add its contents to the next level.
pub fn handle_entry<'a>(
    EntryNode(path, args_left): &'_ EntryNode<'a>,
    found: &'_ mut Vec<PathBuf>,
    new_level: &'_ mut Vec<EntryNode<'a>>,
) {
    // Wrapper to enable the `?` operator.
    let mut inner = || -> Result<(), std::io::Error> {
        match args_left {
            [] => {
                // No args to match left for this path.
                // Add it to found.

                // Last check.
                if path.is_dir() {
                    found.push(path.clone());
                }
            }
            [first_arg, ..] => {
                let read_dir = path
                    .read_dir()?
                    .filter_map(|res| res.ok())
                    .filter(|entry| {
                        entry
                            .file_type()
                            .map(|meta| meta.is_dir())
                            .unwrap_or(false)
                    })
                    .map(|child| child.path());
                let children = read_dir.map(|child| {
                    // If possible, consume the first arg left.
                    if matches(&child, first_arg) {
                        EntryNode(child, &args_left[1..])
                    } else {
                        EntryNode(child, &args_left[..])
                    }
                });

                new_level.extend(children);
            }
        }

        Ok(())
    };

    let _ = inner();
}
