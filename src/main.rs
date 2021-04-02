#![feature(exact_size_is_empty, box_syntax)]

use clap::{App, AppSettings, Arg, SubCommand};
use regex::Regex;
use thiserror::Error;

use std::{
    collections::VecDeque,
    convert::AsRef,
    fs::{read_dir, DirEntry},
    io::Write,
    path::{Path, PathBuf},
    process::exit,
};


/// A slice of path.
pub enum PathSlice {
    /// A slice of path that must be matched right after the previous one.
    Glued(Regex),
    /// A slice of path that can be matched a number of components after the previous one.
    Loose(Regex),
}

/// A result of digging one level further down the file tree.
pub enum DigResult<'a> {
    /// Add path to fully matched paths.
    FullMatch,
    /// End search down that path.
    DeadEnd,
    /// Continue search. Contains all possible paths of traversal from the node.
    Continue(Box<dyn Iterator<Item = EntryNode<'a>> + 'a>),
}

/// A container for an entry and args left to match.
pub struct EntryNode<'a>(pub DirEntry, pub &'a [PathSlice]);

impl<'a> EntryNode<'a> {
    /// Dig one level further down the file tree.
    pub fn dig_deeper<'c>(&self) -> DigResult<'a> {
        let comp = self.0.file_name();
        let comp: &str = &comp.to_string_lossy();

        use DigResult::*;
        let whole = self.1;

        match whole {
            [] => FullMatch,
            [PathSlice::Glued(re), rest @ ..] =>
                if re.is_match(comp) {
                    match rest {
                        [_, ..] => Continue(self.prepare_children(rest)),
                        [] => FullMatch,
                    }
                } else {
                    DeadEnd
                },
            [PathSlice::Loose(re), rest @ ..] =>
                if re.is_match(comp) {
                    match rest {
                        [_, ..] => {
                            // Continue
                            Continue(self.prepare_children(rest))
                        }
                        [] => FullMatch,
                    }
                } else {
                    Continue(self.prepare_children(whole))
                },
        }
    }

    fn prepare_children(
        &self,
        slices_left: &'a [PathSlice],
    ) -> Box<dyn Iterator<Item = EntryNode<'a>> + 'a> {
        let read_dir = match self.0.path().read_dir() {
            Ok(read_dir) => read_dir,
            _ => return box std::iter::empty(),
        };

        let children = read_dir
            .filter_map(|res| res.ok())
            .filter(|entry| {
                entry.file_type().map(|meta| meta.is_dir()).unwrap_or(false)
            })
            .map(move |child_entry| EntryNode(child_entry, slices_left));

        box children
    }
}

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
            prepare_first_level(&start_dir, slices.as_slice()).unwrap_or_else(|_| VecDeque::new());
        let found = find_paths(first_level);

        if let Some(first) = found.get(0) {
            // For now just return the first path found (kinda random). What to do instead?
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
pub fn find_paths<'a>(mut entries: VecDeque<EntryNode<'a>>) -> Vec<PathBuf> {
    // TODO: Use finding depth to get all findings with the same depth
    // and then compare the findings to return the best match (how?).
    let _finding_depth: Option<u32> = None;
    let mut found: Vec<PathBuf> = vec![];

    'entries: while let Some(entry) = entries.pop_front() {
        if !found.is_empty() { break 'entries; }

        use DigResult::*;
        match entry.dig_deeper() {
            FullMatch => found.push(entry.0.path()),
            DeadEnd => { /* do nothing */ },
            Continue(children) => entries.extend(children),
        }
    }

    return found;
}

/// Generate regular expressions from provided args.
pub fn parse_args<'a, A>(args: A) -> Result<(PathBuf, Vec<PathSlice>), Error>
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
        .map(|arg_pattern| {
            Regex::new(arg_pattern).map(|re| PathSlice::Loose(re))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| dev_err!("regex creation"))?;

    Ok((start_dir, args))
}

/// Prepare the first level of [`EntryNode`](crate::EntryNode)s.
pub fn prepare_first_level<'a, P>(
    path: P,
    args: &'a [PathSlice],
) -> Result<VecDeque<EntryNode<'a>>, Error>
where
    P: AsRef<Path>,
{
    let first_level = read_dir(path)?
        .filter_map(|res| res.ok()) // Ignore entires that cannot be accessed.
        .filter(|entry| {
            entry.file_type().map(|meta| meta.is_dir()).unwrap_or(false)
        })
        .map(|entry| {
            EntryNode(entry, args) // TODO: Add field `level` with value 0.
        })
        .collect::<VecDeque<_>>();

    Ok(first_level)
}
