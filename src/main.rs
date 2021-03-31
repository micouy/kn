use regex::Regex;
use std::{
    convert::AsRef,
    fs::{read_dir, DirEntry},
    path::{Path, PathBuf},
};


/// A container for an entry and args left to match.
pub struct EntryNode<'a>(pub PathBuf, pub &'a [Regex]);

fn main() {
    let args = parse_args(std::env::args().skip(1)).unwrap();
    println!("{:?}", args);

    let mut levels: Vec<Vec<EntryNode>> = vec![prepare_first_level(".", &args)];
    let mut found: Vec<PathBuf> = vec![];

    'search: loop {
        let entries = levels.last().unwrap();
        let mut new_level = vec![];

        if entries.is_empty() || !found.is_empty() {
            // either nothing left to search or no need to search.
            break 'search;
        }

        for entry in entries {
            handle_entry(entry, &mut found, &mut new_level);
        }

        levels.push(new_level);
    }

    println!("{:?}", found);
}

/// Generate regular expressions from provided args.
pub fn parse_args<A>(args: A) -> Result<Vec<Regex>, String>
where
    A: Iterator<Item = String>,
{
    let sanitizer = Regex::new("^[[:alnum:]]+$").unwrap();
    let arg_patterns = args
        .map(|arg| {
            if sanitizer.is_match(&arg) {
                Ok(format!("^.*{}.*$", arg))
            } else {
                Err("nieładny string".to_string())
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let args = arg_patterns
        .iter()
        .map(|arg_pattern| Regex::new(arg_pattern))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("{}", err));

    args
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
) -> Vec<EntryNode<'a>>
where
    P: AsRef<Path>,
{
    let entries: Vec<DirEntry> =
        read_dir(path).unwrap().filter_map(|res| res.ok()).collect();
    let arg = &args[0];
    let mut level = vec![];

    for entry in entries {
        let path = entry.path();

        if matches(&path, arg) {
            level.push(EntryNode(path, &args[1..]));
        } else {
            level.push(EntryNode(path, &args[..]));
        }
    }

    level
}

/// Either add entry to found entries or add its contents to the next level.
pub fn handle_entry<'a>(
    EntryNode(path, args_left): &'_ EntryNode<'a>,
    found: &'_ mut Vec<PathBuf>,
    new_level: &'_ mut Vec<EntryNode<'a>>,
) {
    // wrapper to enable the `?` operator
    let mut inner = || -> Result<(), std::io::Error> {
        match args_left {
            [] => {
                // no args to match left for this path.
                // add it to found.
                found.push(path.clone());
            }
            [first_arg, ..] => {
                let read_dir = path
                    .read_dir()?
                    .filter_map(|res| res.ok())
                    .map(|child| child.path());
                let children = read_dir.map(|child| {
                    // if possible, consume the first arg left
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
