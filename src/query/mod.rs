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

#[derive(Debug)]
struct Finding {
    file_name: OsString,
    path: PathBuf,
    congruence: Vec<Congruence>,
}

fn dig<'a, P>(
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

pub fn query(arg: OsString) -> Result<PathBuf, Error> {
    let (prefix, abbrs) = parse_arg(&arg)?;
    let start_dir = match prefix {
        Some(start_dir) => start_dir,
        None => std::env::current_dir()?,
    };

    match abbrs.as_slice() {
        [] => Ok(start_dir),
        [first_abbr, abbrs @ ..] => {
            let mut current_level =
                dig(&start_dir, first_abbr, &[]).collect::<Vec<_>>();
            let mut next_level = vec![];

            for abbr in abbrs {
                let children = current_level
                    .iter()
                    .map(|parent| dig(&parent.path, abbr, &parent.congruence))
                    .flatten();

                next_level.clear();
                next_level.extend(children);

                mem::swap(&mut next_level, &mut current_level);
            }

            let current_level: Vec<_> = current_level;

            let found_path = current_level
                .into_iter()
                .min_by(|finding_a, finding_b| {
                    finding_a.congruence.cmp(&finding_b.congruence).then(
                        compare_os_str(
                            &finding_a.file_name,
                            &finding_b.file_name,
                        ),
                    )
                })
                .map(|Finding { path, .. }| path);

            found_path.ok_or(Error::PathNotFound)
        }
    }
}

fn parse_dots(component: &str) -> Option<usize> {
    component
        .chars()
        .try_fold(
            0,
            |n_dots, c| if c == '.' { Some(n_dots + 1) } else { None },
        )
        .and_then(|n_dots| if n_dots > 1 { Some(n_dots - 1) } else { None })
}

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

fn parse_abbrs<'a, I>(abbrs: I) -> Result<Vec<Abbr>, Error>
where
    I: Iterator<Item = Component<'a>> + 'a,
{
    abbrs
        .map(|component_os| {
            component_os
                .as_os_str()
                .to_os_string()
                .into_string()
                .map_err(|_| Error::NonUnicodeInput)
                .map(|component| Abbr::from_str(&component))
        })
        .collect::<Result<Vec<_>, _>>()
}

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
