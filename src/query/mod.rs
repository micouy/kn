use crate::{
    search::{
        self,
        abbr::{Abbr, Congruence},
        fs::{DefaultFileSystem, FileSystem},
    },
    utils::as_path,
    Error,
    Result,
};

use std::{
    collections::VecDeque,
    ffi::OsStr,
    mem,
    path::{Component, Path, PathBuf},
};

use ansi_term::Colour::Red;
use clap::ArgMatches;
use log::{debug, info, trace};

pub fn query(abbr: &OsStr) -> Result<PathBuf> {
    trace!("Handling query.");

    let file_system = DefaultFileSystem;
    let (start_path, abbr) = parse_args(abbr)?;

    trace!("Start path: `{}`.", start_path.display());

    if abbr.is_empty() {
        trace!("Only starting dir provided, returning.");

        Ok(start_path)
    } else {
        let paths = search::search_full(start_path, abbr.iter(), &file_system);
        let path = paths.get(0).cloned().ok_or(Error::NoPathFound);

        path
    }
}

fn parse_args(abbr: &OsStr) -> Result<(PathBuf, Vec<Abbr>)> {
    trace!("Parsing args.");

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
}
