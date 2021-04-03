//! Utils.

use regex::Regex;

use std::{
    collections::VecDeque,
    convert::AsRef,
    fs::read_dir,
    path::{Path, PathBuf},
};

use crate::{
    error::Error,
    node::{DigResult, EntryNode, PathSlice, PathSlices},
};

/// Find paths matching slices, beginning search at `start_dir`.
pub fn find_paths<'a>(
    mut entries: VecDeque<EntryNode<'a>>,
    _first_max_depth: Option<u32>,
    _next_max_depth: Option<u32>,
) -> Vec<PathBuf> {
    log::trace!("find paths");

    // TODO: Use finding depth to get all findings with the same depth
    // and then compare the findings to return the best match (how?).
    let _finding_depth: Option<u32> = None;
    let mut found: Vec<PathBuf> = vec![];

    'entries: while let Some(entry) = entries.pop_front() {
        if !found.is_empty() {
            break 'entries;
        }

        use DigResult::*;
        match entry.dig_deeper() {
            FullMatch => found.push(entry.0),
            DeadEnd => {
                // do nothing
            }
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
    log::trace!("parse args");

    let mut args = args.peekable();
    if args.peek().is_none() {
        return Err(dev_err!("empty args"));
    }

    // Check if the first arg is a path literal.
    let start_dir = args
        .next_if(|first| AsRef::<Path>::as_ref(first).is_dir())
        .map(|start_dir| Ok(PathBuf::from(start_dir)))
        .unwrap_or_else(std::env::current_dir)?;

    // Check if the slices are valid.
    let alnum_re = Regex::new("^[[:alnum:]]+$").unwrap();
    let dash_re = Regex::new("^-$").unwrap();
    let alnum_validator = |arg: &str| -> bool { alnum_re.is_match(arg) };
    let dash_validator = |arg: &str| -> bool { dash_re.is_match(arg) };
    let generate_re = |arg: &str| -> Result<Regex, Error> {
        let pattern = if alnum_validator(arg) {
            format!("^.*{}.*$", arg)
        } else if dash_validator(arg) {
            "^.+$".to_string()
        } else {
            return Err(Error::InvalidSlice);
        };

        Regex::new(&pattern).map_err(|_| dev_err!("regex creation"))
    };

    let parse_arg = |arg: &str| {
        let mut sub_args = arg.split("/");
        let first = sub_args.next().map(|first| -> Result<PathSlice, Error> {
            let re = generate_re(first)?;

            Ok(PathSlice::Loose(re))
        });
        let rest = sub_args.map(|arg| -> Result<_, _> {
            let re = generate_re(arg)?;

            Ok(PathSlice::Glued(re))
        });

        // Collect, otherwise it irritates the borrow-checker.
        first.into_iter().chain(rest).collect::<Vec<_>>()
    };

    let args = args
        .map(parse_arg)
        .flatten()
        .collect::<Result<Vec<_>, _>>()?;

    Ok((start_dir, args))
}

/// Prepare the first level of [`EntryNode`](crate::node::EntryNode)s.
pub fn prepare_first_level<'a, P>(
    path: P,
    args: &'a [PathSlice],
) -> Result<VecDeque<EntryNode<'a>>, Error>
where
    P: AsRef<Path>,
{
    log::trace!("prepare first level");

    let first_level = read_dir(path)?
        .filter_map(|res| res.ok()) // Ignore entires that cannot be accessed.
        .filter(|entry| {
            entry.file_type().map(|meta| meta.is_dir()).unwrap_or(false)
        })
        .map(|entry| {
            // TODO: Add field `level` with value 0.
            EntryNode(entry.path(), PathSlices::new(args))
        })
        .collect::<VecDeque<_>>();

    Ok(first_level)
}
