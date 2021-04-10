#[allow(dead_code)]
use crate::{utils::as_path, Error, Result};

use std::{
    collections::VecDeque,
    path::{Component, Path, PathBuf},
};


use ansi_term::Colour::Red;
use clap::ArgMatches;
use log::{debug, info, trace};


mod abbr;
mod entry;
mod search_engine;


use abbr::Abbr;
use entry::Entry;
use search_engine::{ReadDirEngine, SearchEngine};


pub fn query(matches: &ArgMatches<'_>) -> Result<Vec<PathBuf>> {
    trace!("Handling query.");


    let engine = ReadDirEngine;
    let (start_path, abbr) = parse_args(matches)?;

    trace!("Start path: `{}`.", start_path.display());

    match abbr.as_slice() {
        [first, rest @ ..] => search(start_path, first, rest, &engine),
        [] => {
            trace!("Only starting dir provided, returning.");


            return Ok(vec![start_path]);
        }
    }
}


fn search<P, E>(
    start_path: P,
    abbr: &Abbr,
    rest: &[Abbr],
    engine: E,
) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
    E: SearchEngine,
{
    use entry::Flow::*;


    let mut queue = engine
        .read_dir(start_path)
        .into_iter()
        .map(|subdir| Entry::new(subdir, abbr, rest))
        .collect::<Result<VecDeque<_>>>()?;


    let mut found: Option<(usize, Vec<Entry>)> = None;

    while let Some(entry) = queue.pop_front() {
        // Reject entries that are deeper than the ones in `found`.
        if let Some((depth, _)) = found {
            if entry.n_attempts() > depth {
                continue;
            }
        }

        match entry.advance(&engine) {
            DeadEnd => {
                debug!(
                    "Dead end `{}`.",
                    Red.paint(entry.path().to_string_lossy())
                );
            }
            Continue(children) => {
                info!("Continue down `{}`.", entry.path().display());
                queue.extend(children.into_iter());
            }
            FullMatch(entry) => {
                info!("Full match `{}`.", entry.path().display());

                // Update `found`.
                match found {
                    Some((_, ref mut entries)) => entries.push(entry),
                    None => found = Some((entry.n_attempts(), vec![entry])),
                }
            }
        }
    }


    match found {
        Some((_, entries)) => {
            // TODO: Return an object containing details about matches?
            trace!("Found entries:");

            for entry in &entries {
                trace!("Path: `{}`.", entry.path().display());
                trace!("Congruence: `{:?}`.", entry.congruence());
            }


            let paths = get_ordered_paths(entries);

            Ok(paths)
        }
        None => Err(Error::NoPathFound),
    }
}


fn get_ordered_paths(mut entries: Vec<Entry<'_>>) -> Vec<PathBuf> {
    entries.sort_by(|a, b| a.congruence().cmp(b.congruence()));

    let paths = entries.into_iter().map(|entry| entry.path).collect();

    paths
}


fn parse_args(matches: &ArgMatches<'_>) -> Result<(PathBuf, Vec<Abbr>)> {
    trace!("Parsing args.");

    let abbr = matches
        .value_of_os("ABBR")
        .ok_or(dev_err!("required `clap` arg absent"))?;
    let abbr = abbr.to_str().ok_or(Error::ArgInvalidUnicode)?;

    if abbr.is_empty() {
        return Err(Error::EmptyAbbr);
    }

    let (start_path, suffix) = extract_start_path(as_path(abbr));

    let start_path = start_path
        .map(|path| Ok(path))
        .unwrap_or_else(|| std::env::current_dir())?;

    let abbr = suffix
        .into_iter()
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .ok_or(Error::ArgInvalidUnicode)
                .and_then(|s| Abbr::from_string(s.to_string()))
        })
        .collect::<Result<Vec<Abbr>>>()?;

    if let Some(Abbr::Wildcard) = abbr.last() {
        return Err(Error::WildcardAtLastPlace);
    }

    trace!("Abbreviation `{:?}`.", abbr);


    Ok((start_path, abbr))
}


fn extract_start_path<'p>(
    arg: &'p Path,
) -> (Option<PathBuf>, Vec<Component<'p>>) {
    trace!("Extracting start path.");


    let mut suffix = arg.components().peekable();
    let mut prefix: Option<PathBuf> = None;


    // Handle cases `kn /**/*`, `kn C:/**/*`, `kn ../../**/*` etc..
    // Doesn't handle a literal tilde, it must be expanded by the shell.
    while let Some(component) = suffix.next_if(|component| {
        use std::path::Component::*;
        match component {
            Prefix(_) | RootDir | CurDir | ParentDir => true,
            Normal(_) => false,
        }
    }) {
        match prefix {
            Some(ref mut prefix) => prefix.push(component),
            None => {
                prefix = Some(PathBuf::from(as_path(&component)));
            }
        }
    }

    let suffix = suffix.collect();

    (prefix, suffix)
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::as_path;

    use std::collections::HashMap;

    use pretty_assertions::assert_eq;


    #[test]
    fn test_extract_start_path() {
        // No start path.
        let (start_path, suffix) = extract_start_path(as_path("a/b/c"));
        let first_abbr = suffix[0].as_os_str();

        assert!(start_path.is_none());
        assert_eq!(first_abbr, "a");


        // Root dir.
        let (start_path, suffix) = extract_start_path(as_path("/gn"));
        let first_abbr = suffix[0].as_os_str();

        assert_eq!(start_path.unwrap(), as_path("/"));
        assert_eq!(first_abbr, "gn");


        // Multiple `..` and `.`.
        let (start_path, suffix) = extract_start_path(as_path(".././../do"));
        let first_abbr = suffix[0].as_os_str();

        assert_eq!(start_path.unwrap(), as_path(".././.."));
        assert_eq!(first_abbr, "do");
    }


    #[test]
    fn test_entry_walk() {
        use abbr::Congruence::*;
        use entry::Flow::*;


        // Test path: `a/b`.
        let mut search_engine = HashMap::new();

        search_engine.insert("a".into(), vec!["a/boo".into()]);
        search_engine.insert("a/boo".into(), vec![]);

        let abbr = Abbr::from_string("a".to_string()).unwrap();
        let rest = vec![Abbr::from_string("b".to_string()).unwrap()];


        // The square brackets indicate which component will be matched against
        // which abbreviation.


        // path: [a]
        // slices: [a]/b
        let entry_a = Entry::new("a".into(), &abbr, &rest).unwrap();
        assert_eq!(entry_a.path(), as_path("a"));
        let result = entry_a.advance(&search_engine);


        // path: a/[boo]
        // slices: a/[b]
        let entry_ab =
            variant!(result, Continue(children) => children[0].clone());
        assert_eq!(entry_ab.path(), as_path("a/boo"));
        let result = entry_ab.advance(&search_engine);

        let entry_ab = variant!(result, FullMatch(entry_ab) => entry_ab);
        variant!(entry_ab.congruence(), [Complete, Partial(_)]);
    }


    #[test]
    fn test_dead_end() {
        use entry::Flow::*;


        // Test path: `a/b`.
        let mut search_engine = HashMap::new();
        search_engine.insert("a".into(), vec!["a/o".into()]);
        search_engine.insert("a/o".into(), vec![]);

        let abbr = Abbr::from_string("a".to_string()).unwrap();
        let rest = vec![Abbr::from_string("b".to_string()).unwrap()];


        // path: [a]
        // slices: [a]/b
        let entry_a = Entry::new("a".into(), &abbr, &rest).unwrap();
        assert_eq!(entry_a.path(), as_path("a"));
        let result = entry_a.advance(&search_engine);


        // path: a/[b]
        // slices: a/[b]
        let entry_ab =
            variant!(result, Continue(children) => children[0].clone());
        assert_eq!(entry_ab.path(), as_path("a/o"));
        let result = entry_ab.advance(&search_engine);

        variant!(result, DeadEnd);
    }
}
