#[allow(dead_code)]
use crate::{utils::as_path, Error, Result};

use std::{collections::VecDeque, path::PathBuf};

use clap::ArgMatches;

#[cfg(feature = "logging")]
use ansi_term::Colour::{Green, Red};
#[cfg(feature = "logging")]
use log::{debug, info};

mod entry;
mod search_engine;
mod sequence;
mod slice;

use entry::Entry;
use search_engine::SearchEngine;
use sequence::Sequence;

pub fn query(matches: &ArgMatches<'_>) -> Result<Vec<PathBuf>> {
    let (opts, slices) = parse_args(matches)?;

    if slices.is_empty() {
        return Ok(vec![opts.start_dir]);
    }

    search(opts, slices)
}

fn search(opts: SearchOpts, sequences: Vec<Sequence>) -> Result<Vec<PathBuf>> {
    use entry::EntryMatch::*;

    let engine = search_engine::ReadDirEngine;

    let mut queue = engine
        .read_dir(&opts.start_dir)
        .into_iter()
        .map(|subdir| Entry::new(subdir, sequences.clone(), opts.clone()))
        .collect::<VecDeque<_>>();

    while let Some(entry) = queue.pop_front() {
        match entry.fire_walk(&engine)? {
            DeadEnd => {
                #[cfg(feature = "logging")]
                debug!(
                    "Dead end `{}`.",
                    Red.paint(entry.path().to_string_lossy())
                );
            }
            #[cfg(feature = "logging")]
            Advancement(children, strength) => {
                use MatchStrength::*;

                match strength {
                    Complete | Partial => {
                        info!(
                            "Advancement `{}`.",
                            crate::utils::paint_file_name(
                                entry.path().into(),
                                Green
                            )
                        );
                    }
                    Naught => {
                        info!(
                            "Advancement `{}`.",
                            crate::utils::paint_file_name(
                                entry.path().into(),
                                Red
                            )
                        );
                    }
                }

                queue.extend(children.into_iter());
            }
            #[cfg(not(feature = "logging"))]
            Advancement(children, _) => {
                queue.extend(children.into_iter());
            }
            FullMatch(path, _strength) => {
                #[cfg(feature = "logging")]
                info!("Full match `{}`.", entry.path().display());

                // TODO: Push fully matched entries to `found`. Track depths
                // and reject entries deeper than the ones in `found`.
                return Ok(vec![path]);
            }
        }
    }

    Err(Error::NoPathFound)
}


#[derive(Debug, Clone, Default)]
pub struct SearchOpts {
    first_depth: Option<usize>,
    next_depth: Option<usize>,
    start_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub enum MatchStrength {
    Complete,
    Partial,
    Naught,
}

fn parse_args(matches: &ArgMatches<'_>) -> Result<(SearchOpts, Vec<Sequence>)> {
    let mut slices = matches
        .values_of("SLICES")
        .ok_or(dev_err!("required `clap` arg absent"))?
        .peekable();

    if slices.is_empty() {
        return Err(dev_err!("required `clap` arg empty"));
    }

    let start_dir = slices
        .next_if(|first| as_path(first).is_dir())
        .map(|first| Ok(PathBuf::from(first)))
        .unwrap_or_else(|| std::env::current_dir())?;

    let slices = slices
        .map(|slice| Sequence::from_str(slice))
        .collect::<Result<Vec<_>>>()?;

    let first_depth = matches
        .value_of("first-max-depth")
        .map(|depth| depth.parse::<usize>())
        .transpose()
        .map_err(|_| Error::InvalidArgValue("first-max-depth".to_string()))?;

    let next_depth = matches
        .value_of("next-max-depth")
        .map(|depth| depth.parse::<usize>())
        .transpose()
        .map_err(|_| Error::InvalidArgValue("next-max-depth".to_string()))?;

    let opts = SearchOpts {
        first_depth,
        next_depth,
        start_dir,
    };

    Ok((opts, slices))
}

#[cfg(test)]
mod test {
    use super::{entry::EntryMatch, *};
    use crate::utils::as_path;

    use std::collections::HashMap;

    // TODO: Add tests with multiple sequences and different options.

    #[test]
    fn test_entry_walk() {
        use EntryMatch::*;
        use MatchStrength::*;

        let sequence_ab: Sequence = Sequence::from_str("a/b").unwrap();
        let opts = SearchOpts::default();

        // Test path: `a/o/a/b`.
        let mut search_engine = HashMap::new();
        search_engine.insert("a".into(), vec!["a/o".into()]);
        search_engine.insert("a/o".into(), vec!["a/o/a".into()]);
        search_engine.insert("a/o/a".into(), vec!["a/o/a/b".into()]);

        // The square brackets indicate which component will be matched against
        // which slice.

        // path: [a]
        // slices: [a]/b
        let entry_a = Entry::new("a".into(), vec![sequence_ab], opts);
        assert_eq!(entry_a.path(), as_path("a"));
        let result = entry_a.fire_walk(&search_engine).unwrap();

        // path: a/[o]
        // slices: a/[b]
        let entry_ao = variant!(result, Advancement(children, Complete) => children[0].clone());
        assert_eq!(entry_ao.path(), as_path("a/o"));
        let result = entry_ao.fire_walk(&search_engine).unwrap();

        // path: a/o/[a]
        // slices: [a]/b
        let entry_aoa = variant!(result, Advancement(children, Naught) => children[0].clone());
        assert_eq!(entry_aoa.path(), as_path("a/o/a"));
        let result = entry_aoa.fire_walk(&search_engine).unwrap();

        // path: a/o/a/[b]
        // slices: a/[b]
        let entry_aoab = variant!(result, Advancement(children, Complete) => children[0].clone());
        assert_eq!(entry_aoab.path(), as_path("a/o/a/b"));
        let result = entry_aoab.fire_walk(&search_engine).unwrap();

        let path = variant!(result, FullMatch(path, Complete) => path);
        assert_eq!(path, as_path("a/o/a/b"));
    }

    #[test]
    fn test_zero_first_depth() {
        let search_engine = HashMap::new();
        let opts = SearchOpts {
            first_depth: Some(0),
            ..Default::default()
        };

        let sequence_a: Sequence = Sequence::from_str("a").unwrap();
        let entry = Entry::new("o".into(), vec![sequence_a], opts);

        // Check slice `a` against path `o` with `first_depth` set to 0.
        let result = entry.fire_walk(&search_engine).unwrap();
        variant!(result, EntryMatch::DeadEnd => ());
    }

    #[test]
    fn test_premature_match_dead_end() {
        use EntryMatch::*;

        let sequence_abc: Sequence = Sequence::from_str("a/b/c").unwrap();
        let opts = SearchOpts {
            first_depth: Some(1),
            ..Default::default()
        };

        // Test path: `a/b/o`.
        let mut search_engine = HashMap::new();
        search_engine.insert("a".into(), vec!["a/b".into()]);
        search_engine.insert("a/b".into(), vec!["a/b/o".into()]);

        // path: [a]
        // slices: [a]/b/c
        let entry_a = Entry::new("a".into(), vec![sequence_abc], opts);
        let result = entry_a.fire_walk(&search_engine).unwrap();

        // path: a/[b]
        // slices: a/[b]/c
        let entry_ab =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_ab.fire_walk(&search_engine).unwrap();

        // path: a/b/o
        // slices: a/b/[c]
        let entry_abo =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_abo.fire_walk(&search_engine).unwrap();
        variant!(result, DeadEnd => ());

        // No matter what the continuation of this path is (`a/b/o/*`), the
        // options would be violated in the subsequent matches.
    }

    #[test]
    fn test_premature_match_recovery() {
        use EntryMatch::*;

        let sequence_abc: Sequence = Sequence::from_str("a/b").unwrap();
        let opts = SearchOpts {
            first_depth: Some(2),
            ..Default::default()
        };

        // Test path: `a/o/a/b`.
        let mut search_engine = HashMap::new();
        search_engine.insert("a".into(), vec!["a/o".into()]);
        search_engine.insert("a/o".into(), vec!["a/o/a".into()]);
        search_engine.insert("a/o/a".into(), vec!["a/o/a/b".into()]);

        // path: [a]
        // slices: [a]/b
        let entry_a = Entry::new("a".into(), vec![sequence_abc], opts);
        let result = entry_a.fire_walk(&search_engine).unwrap();

        // path: a/[o]
        // slices: a/[b]
        let entry_ao =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_ao.fire_walk(&search_engine).unwrap();

        // Entry `a/o` doesn't return `DeadEnd`, because there might still be a
        // match for slice `a` at index 2 - in the next entry `a/o/*`
        // component * might match 'a'.

        // path: a/o/[a]
        // slices: [a]/b
        let _entry_aoa =
            variant!(result, Advancement(children, _) => children[0].clone());
    }

    #[test]
    fn test_wildcard() {
        use EntryMatch::*;

        let opts = SearchOpts {
            first_depth: Some(0),
            next_depth: Some(0),
            ..Default::default()
        };

        let mut search_engine = HashMap::new();
        search_engine.insert("a".into(), vec!["a/o".into()]);
        search_engine.insert("a/o".into(), vec!["a/o/b".into()]);

        // path: [a]
        // slices: [a]/-/b
        let sequence: Sequence = Sequence::from_str("a/-/b").unwrap();
        let entry_a = Entry::new("a".into(), vec![sequence], opts);
        let result = entry_a.fire_walk(&search_engine).unwrap();

        // Wildcard matches any (every?) component.
        // path: a/o
        // slices: a/[-]/b
        let entry_ao =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_ao.fire_walk(&search_engine).unwrap();

        // path: a/o/b
        // slices: a/-/[b]
        let entry_aob =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_aob.fire_walk(&search_engine).unwrap();

        variant!(result, FullMatch(_, _));
    }
}
