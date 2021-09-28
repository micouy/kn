use crate::{
    abbr::{Abbr, Congruence},
    error::Error,
};

use std::{
    convert::AsRef,
    ffi::{OsStr, OsString},
    fs::DirEntry,
    mem,
    path::{Component, Path, PathBuf},
};

use alphanumeric_sort::compare_os_str;

/// A path matching an abbreviation.
///
/// Stores [`Congruence`](Congruence)'s of its ancestors, with that of the
/// closest ancestors first (so that it can be compared
/// [lexicographically](std::cmp::Ord#lexicographical-comparison).
struct Finding {
    file_name: OsString,
    path: PathBuf,
    congruence: Vec<Congruence>,
}

/// Returns an interator over directory's children matching the abbreviation.
fn get_matching_children<'a, P>(
    path: &'a P,
    abbr: &'a Abbr,
    parent_congruence: &'a [Congruence],
) -> impl Iterator<Item = Finding> + 'a
where
    P: AsRef<Path>,
{
    let filter_map_entry = move |entry: DirEntry| {
        let file_type = entry.file_type().ok()?;

        if file_type.is_dir() || file_type.is_symlink() {
            let file_name: String = entry.file_name().into_string().ok()?;

            if let Some(congruence) = abbr.compare(&file_name) {
                let mut entry_congruence = parent_congruence.to_vec();
                entry_congruence.insert(0, congruence);

                return Some(Finding {
                    file_name: entry.file_name(),
                    congruence: entry_congruence,
                    path: entry.path(),
                });
            }
        }

        None
    };

    path.as_ref()
        .read_dir()
        .ok()
        .map(|reader| {
            reader
                .filter_map(|entry| entry.ok())
                .filter_map(filter_map_entry)
        })
        .into_iter()
        .flatten()
}

/// The `query` subcommand.
///
/// The provided arg gets split into a prefix and [`Abbr`](Abbr)'s.
/// The prefix is the path where the search starts. See
/// [`extract_prefix`](extract_prefix).
pub fn query<P>(arg: &P, excluded: Option<PathBuf>) -> Result<PathBuf, Error>
where
    P: AsRef<Path>,
{
    // If the arg is a real path and not an abbreviation, return it. It
    // prevents potential unexpected behavior due to abbreviation expansion.
    // For example, `kn` doesn't allow for any component other than `Normal` in
    // the abbreviation but the arg itself may be a valid path. `kn` should only
    // behave differently from `cd` in situations where `cd` would fail.
    if arg.as_ref().is_dir() {
        return Ok(arg.as_ref().into());
    }

    let (prefix, abbrs) = parse_arg(&arg)?;
    let start_dir = match prefix {
        Some(start_dir) => start_dir,
        None => std::env::current_dir()?,
    };

    match abbrs.as_slice() {
        [] => Ok(start_dir),
        [first_abbr, abbrs @ ..] => {
            let mut current_level =
                get_matching_children(&start_dir, first_abbr, &[])
                    .collect::<Vec<_>>();
            let mut next_level = vec![];

            for abbr in abbrs {
                let children = current_level
                    .iter()
                    .map(|parent| {
                        get_matching_children(
                            &parent.path,
                            abbr,
                            &parent.congruence,
                        )
                    })
                    .flatten();

                next_level.clear();
                next_level.extend(children);

                mem::swap(&mut next_level, &mut current_level);
            }

            let cmp_findings = |finding_a: &Finding, finding_b: &Finding| {
                finding_a.congruence.cmp(&finding_b.congruence).then(
                    compare_os_str(&finding_a.file_name, &finding_b.file_name),
                )
            };

            let found_path = match excluded {
                Some(excluded) if current_level.len() > 1 => current_level
                    .into_iter()
                    .filter(|finding| finding.path != excluded)
                    .min_by(cmp_findings)
                    .map(|Finding { path, .. }| path),
                _ => current_level
                    .into_iter()
                    .min_by(cmp_findings)
                    .map(|Finding { path, .. }| path),
            };

            found_path.ok_or(Error::PathNotFound)
        }
    }
}

/// Checks if the component contains only dots and returns the equivalent number
/// of [`ParentDir`](Component::ParentDir) components if it does.
///
/// It is the number of dots, less one. For example, `...` is converted to
/// `../..`, `....` to `../../..` etc.
fn parse_dots(component: &str) -> Option<usize> {
    component
        .chars()
        .try_fold(
            0,
            |n_dots, c| if c == '.' { Some(n_dots + 1) } else { None },
        )
        .and_then(|n_dots| if n_dots > 1 { Some(n_dots - 1) } else { None })
}

/// Extracts leading components of the path that are not parts of the
/// abbreviation.
///
/// The prefix is the path where the search starts. If there is no prefix (when
/// the path consists only of normal components), the search starts in the
/// current directory, just as you'd expect. The function collects each
/// [`Prefix`](Component::Prefix), [`RootDir`](Component::RootDir),
/// [`CurDir`](Component::CurDir), and [`ParentDir`](Component::ParentDir)
/// components and stops at the first [`Normal`] component **unless** it only
/// contains dots. In this case, it converts it to as many
/// [`ParentDir`](Component::ParentDir)'s as there are dots in this component,
/// less one. For example, `...` is converted to `../..`, `....` to `../../..`
/// etc.
fn extract_prefix<'a, P>(
    arg: &'a P,
) -> Result<(Option<PathBuf>, impl Iterator<Item = Component<'a>> + 'a), Error>
where
    P: AsRef<Path> + ?Sized + 'a,
{
    use Component::*;

    let mut components = arg.as_ref().components().peekable();
    let mut prefix: Option<PathBuf> = None;
    let mut push_to_prefix = |component: Component| match &mut prefix {
        None => prefix = Some(PathBuf::from(&component)),
        Some(prefix) => prefix.push(component),
    };
    let parse_dots_os = |component_os: &OsStr| {
        component_os
            .to_os_string()
            .into_string()
            .map_err(|_| Error::NonUnicodeInput)
            .map(|component| parse_dots(&component))
    };

    while let Some(component) = components.peek() {
        match component {
            Prefix(_) | RootDir | CurDir | ParentDir =>
                push_to_prefix(*component),
            Normal(component_os) => {
                if let Some(n_dots) = parse_dots_os(component_os)? {
                    (0..n_dots).for_each(|_| push_to_prefix(ParentDir));
                } else {
                    break;
                }
            }
        }

        let _consumed = components.next();
    }

    Ok((prefix, components))
}

/// Converts each component into [`Abbr`](Abbr) without checking
/// the component's type.
///
/// This may change in the future.
fn parse_abbrs<'a, I>(components: I) -> Result<Vec<Abbr>, Error>
where
    I: Iterator<Item = Component<'a>> + 'a,
{
    use Component::*;

    let abbrs = components
        .into_iter()
        .map(|component| match component {
            Prefix(_) | RootDir | CurDir | ParentDir => {
                let component_string = component
                    .as_os_str()
                    .to_os_string()
                    .to_string_lossy()
                    .to_string();

                Err(Error::UnexpectedAbbrComponent(component_string))
            }
            Normal(component_os) => component_os
                .to_os_string()
                .into_string()
                .map_err(|_| Error::NonUnicodeInput)
                .map(|string| Abbr::from_str(&string)),
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(abbrs)
}

/// Parses the provided argument into a prefix and [`Abbr`](Abbr)'s.
fn parse_arg<P>(arg: &P) -> Result<(Option<PathBuf>, Vec<Abbr>), Error>
where
    P: AsRef<Path>,
{
    let (prefix, suffix) = extract_prefix(arg)?;
    let abbrs = parse_abbrs(suffix)?;

    Ok((prefix, abbrs))
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::utils::as_path;

    #[test]
    fn test_parse_dots() {
        assert_variant!(parse_dots(""), None);
        assert_variant!(parse_dots("."), None);
        assert_variant!(parse_dots(".."), Some(1));
        assert_variant!(parse_dots("..."), Some(2));
        assert_variant!(parse_dots("...."), Some(3));
        assert_variant!(parse_dots("xyz"), None);
        assert_variant!(parse_dots("...dot"), None);
    }

    #[test]
    fn test_extract_prefix() {
        {
            let (prefix, suffix) = extract_prefix("suf/fix").unwrap();
            let suffix = suffix.collect::<PathBuf>();

            assert_eq!(prefix, None);
            assert_eq!(as_path(&suffix), as_path("suf/fix"));
        }

        {
            let (prefix, suffix) = extract_prefix("./.././suf/fix").unwrap();
            let suffix = suffix.collect::<PathBuf>();

            assert_eq!(prefix.unwrap(), as_path("./.."));
            assert_eq!(as_path(&suffix), as_path("suf/fix"));
        }

        {
            let (prefix, suffix) = extract_prefix(".../.../suf/fix").unwrap();
            let suffix = suffix.collect::<PathBuf>();

            assert_eq!(prefix.unwrap(), as_path("../../../.."));
            assert_eq!(as_path(&suffix), as_path("suf/fix"));
        }
    }

    #[test]
    fn test_parse_arg_invalid_unicode() {
        #[cfg(unix)]
        {
            use std::ffi::OsStr;
            use std::os::unix::ffi::OsStrExt;

            let source = [0x66, 0x6f, 0x80, 0x6f];
            let non_unicode_input =
                OsStr::from_bytes(&source[..]).to_os_string();
            let result = parse_arg(&non_unicode_input);

            assert!(result.is_err());
        }

        #[cfg(windows)]
        {
            use std::os::windows::prelude::*;

            let source = [0x0066, 0x006f, 0xD800, 0x006f];
            let os_string = OsString::from_wide(&source[..]);
            let result = parse_arg(&non_unicode_input);

            assert!(result.is_err());
        }
    }
}
