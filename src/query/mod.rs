use crate::{abbr::Abbr, error::Error};

use std::{
    convert::AsRef,
    ffi::{OsStr, OsString},
    path::{Component, Path, PathBuf},
};

pub fn query(arg: OsString) -> Result<PathBuf, Error> {
    let (prefix, abbrs) = parse_arg(&arg)?;
    let start_dir = match prefix {
        Some(start_dir) => start_dir,
        None => std::env::current_dir()?,
    };

    todo!("search the file system")
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
