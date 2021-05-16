use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};

use crate::{
    search::{self, abbr::Abbr, fs::DefaultFileSystem},
    utils::as_path,
    Error,
    Result,
};

pub fn query(abbr: &OsStr) -> Result<PathBuf> {
    let file_system = DefaultFileSystem;
    let (start_path, abbr) = parse_args(abbr)?;

    if abbr.is_empty() {
        Ok(start_path)
    } else {
        let paths = search::search_full(start_path, abbr.iter(), &file_system);
        let path = paths.get(0).cloned().ok_or(Error::NoPathFound);

        path
    }
}

fn parse_args(abbr: &OsStr) -> Result<(PathBuf, Vec<Abbr>)> {
    if abbr.is_empty() {
        return Err(Error::EmptyAbbr);
    }

    let (start_path, suffix) = decompose_arg(as_path(abbr))?;

    let start_path = start_path
        .map(|path| Ok(path))
        .unwrap_or_else(|| std::env::current_dir())?;

    let abbr = suffix
        .into_iter()
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .ok_or(Error::InvalidUnicode)
                .and_then(|s| Abbr::from_string(s.to_string()))
        })
        .collect::<Result<Vec<Abbr>>>()?;

    if let Some(Abbr::Wildcard) = abbr.last() {
        return Err(Error::WildcardAtLastPlace);
    }

    Ok((start_path, abbr))
}

fn maybe_parse_dots(component: &str) -> Option<u32> {
    component.chars()
    	.try_fold(0, |occurences, c| if c == '.' { Some(occurences + 1) } else { None })
        .and_then(|occurences| if occurences >= 1 {
            Some(occurences - 1)
        } else {
            None
        })
}

fn decompose_arg<'a, P>(arg: &'a P) -> Result<(Option<PathBuf>, Vec<Component<'a>>)>
where P: AsRef<Path> + ?Sized + 'a {
    use std::path::Component::*;

    let arg = arg.as_ref();
    let mut arg = arg.components().peekable();
    let mut prefix: Option<PathBuf> = None;

    let mut push_to_prefix = |component| {
        match prefix {
            Some(ref mut prefix) => prefix.push(component),
            None => {
                prefix = Some(PathBuf::from(as_path(&component)));
            }
        }
    };

    while let Some(component) = arg.peek() {
        match component {
            Prefix(_) | RootDir | CurDir | ParentDir => push_to_prefix(component.clone()),
            Normal(component_os) => {
                let component = component_os.to_str().ok_or(Error::InvalidUnicode)?;

                if let Some(n_dots) = maybe_parse_dots(component) {
                    for _ in 0..n_dots {
                    	push_to_prefix(ParentDir);
                    }
                } else {
                    break;
                }
            },
        }

        arg.next();
    }

    let arg = arg.collect();

    Ok((prefix, arg))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::as_path;

    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_decompose_arg() {
        // No start path.
        let (start_path, suffix) = decompose_arg(as_path("a/b/c")).unwrap();
        let first_abbr = suffix[0].as_os_str();

        assert!(start_path.is_none());
        assert_eq!(first_abbr, "a");

        // Root dir.
        let (start_path, suffix) = decompose_arg(as_path("/gn")).unwrap();
        let first_abbr = suffix[0].as_os_str();

        assert_eq!(start_path.unwrap(), as_path("/"));
        assert_eq!(first_abbr, "gn");

        // Multiple `..` and `.`.
        let (start_path, suffix) = decompose_arg(as_path(".././../do")).unwrap();
        let first_abbr = suffix[0].as_os_str();

        assert_eq!(start_path.unwrap(), as_path(".././.."));
        assert_eq!(first_abbr, "do");

        // Three or more dots.
        let (start_path, suffix) = decompose_arg(as_path("./../.../..../oops")).unwrap();
        let first_abbr = suffix[0].as_os_str();

        // . = 0
        // .. = 1
        // ... = 2
        // .... = 3
        // total of 6
        assert_eq!(start_path.unwrap(), as_path("./../../../../../../"));
        assert_eq!(first_abbr, "oops");
    }
}
