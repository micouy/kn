#[allow(dead_code)]
use crate::{Error, Result};

use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

use clap::ArgMatches;

use ansi_term::Colour::{Green, Red};
use log::{debug, info};

mod entry;
mod search_engine;
mod sequence;
mod slice;

use entry::Entry;
use search_engine::{ReadDirEngine, SearchEngine};
use sequence::Sequence;
use slice::Slice;

pub fn query(matches: &ArgMatches<'_>) -> Result<Vec<PathBuf>> {
    let engine = ReadDirEngine;
    let (opts, sequences) = parse_args(matches, &engine)?;

    if sequences.is_empty() {
        return Ok(vec![opts.start_dir]);
    }

    search(sequences, opts, &engine)
}

fn search<E>(
    sequences: Vec<Sequence>,
    opts: SearchOpts,
    engine: E,
) -> Result<Vec<PathBuf>>
where
    E: SearchEngine,
{
    use entry::EntryMatch::*;

    let mut queue = engine
        .read_dir(&opts.start_dir)
        .into_iter()
        .map(|subdir| Entry::new(subdir, sequences.clone(), opts.clone()))
        .collect::<VecDeque<_>>();

    while let Some(entry) = queue.pop_front() {
        match entry.fire_walk(&engine)? {
            DeadEnd => {
                debug!(
                    "Dead end `{}`.",
                    Red.paint(entry.path().to_string_lossy())
                );
            }
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
            FullMatch(path, _strength) => {
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

fn parse_args<E>(
    matches: &ArgMatches<'_>,
    engine: E,
) -> Result<(SearchOpts, Vec<Sequence>)>
where
    E: SearchEngine,
{
    let mut sequences = matches
        .values_of("SLICES")
        .ok_or(dev_err!("required `clap` arg absent"))?;

    let first_arg = sequences
        .next()
        .ok_or(dev_err!("required `clap` arg empty"))?;
    let (start_dir, mb_first_sequence) = extract_start_dir(first_arg, engine)?;

    let sequences = sequences.map(|slice| Sequence::from_str(slice));
    let sequences = mb_first_sequence
        .map(|sequence| Ok(sequence))
        .into_iter()
        .chain(sequences)
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

    Ok((opts, sequences))
}

fn extract_start_dir<E, P>(
    path: P,
    engine: E,
) -> Result<(PathBuf, Option<Sequence>)>
where
    P: AsRef<Path>,
    E: SearchEngine,
{
    // TODO: Remove repetition.

    use std::path::Component::*;

    let mut prefix = path.as_ref().components();
    let mut rejected = VecDeque::new();

    loop {
        let subpath = prefix.as_path();

        if engine.is_dir(subpath) {
            let start_dir = subpath.into();
            let slices = rejected
                .into_iter()
                .map(|component| match component {
                    Normal(os_str) => {
                        let string = os_str.to_string_lossy().into_owned();

                        Slice::from_string(string)
                    }
                    _ => Err(Error::InvalidSlice(format!(
                        "{:?}",
                        component.as_os_str()
                    ))),
                })
                .collect::<Result<Vec<_>>>()?;
            let sequence = if slices.is_empty() {
                None
            } else {
                Some(Sequence {
                    slices,
                    slice_to_match: 0,
                })
            };

            return Ok((start_dir, sequence));
        }

        match prefix.next_back() {
            Some(component) => rejected.push_front(component),
            None => break,
        }
    }

    let start_dir = std::env::current_dir()?;
    let slices =
        rejected
            .into_iter()
            .map(|component| match component {
                Normal(os_str) => {
                    let string = os_str.to_string_lossy().into_owned();

                    Slice::from_string(string)
                }
                _ => Err(Error::InvalidSlice(format!(
                    "{:?}",
                    component.as_os_str()
                ))),
            })
            .collect::<Result<Vec<_>>>()?;
    let sequence = if slices.is_empty() {
        None
    } else {
        Some(Sequence {
            slices,
            slice_to_match: 0,
        })
    };

    return Ok((start_dir, sequence));
}

#[cfg(test)]
mod test {
    use super::{entry::EntryMatch, slice::Slice, *};
    use crate::utils::as_path;

    use EntryMatch::*;

    use std::collections::HashMap;

    #[test]
    fn test_extract_start_dir() {
        let mut search_engine = HashMap::new();

        search_engine.insert("ax".into(), vec!["ox".into()]);
        search_engine.insert("ax/ox".into(), vec![]);
        let (start_dir, first) =
            extract_start_dir("ax/ox/ex", &search_engine).unwrap();
        let first = first.unwrap();

        assert_eq!(start_dir, as_path("ax/ox"));
        variant!(&first.slices()[0], Slice::Literal(literal) if literal == "ex");

        search_engine.insert("/".into(), vec!["gniazdo-os".into()]);
        search_engine.insert("/gniazdo-os".into(), vec![]);
        let (start_dir, first) =
            extract_start_dir("/gn", &search_engine).unwrap();
        let first = first.unwrap();

        assert_eq!(start_dir, as_path("/"));
        variant!(&first.slices()[0], Slice::Literal(literal) if literal == "gn");

        search_engine.insert("~".into(), vec!["dodo".into()]);
        search_engine.insert("~/dodo".into(), vec![]);
        let (start_dir, first) =
            extract_start_dir("~/do", &search_engine).unwrap();
        let first = first.unwrap();

        assert_eq!(start_dir, as_path("~"));
        variant!(&first.slices()[0], Slice::Literal(literal) if literal == "do");

        let (start_dir, first) =
            extract_start_dir("~/dodo", &search_engine).unwrap();

        assert_eq!(start_dir, as_path("~/dodo"));
        assert!(first.is_none());
    }

    #[test]
    fn test_entry_walk() {
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
    fn test_first_depth_exceeded() {
        let search_engine = HashMap::new();
        let opts = SearchOpts {
            first_depth: Some(0),
            ..Default::default()
        };

        let sequence_a: Sequence = Sequence::from_str("a").unwrap();
        let entry = Entry::new("o".into(), vec![sequence_a], opts);

        // Check slice `a` against path `o` with `first_depth` set to 0.
        let result = entry.fire_walk(&search_engine).unwrap();
        variant!(result, DeadEnd);
    }

    #[test]
    fn test_premature_match() {
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

    #[test]
    fn test_multiple_sequences() {
        let opts = SearchOpts {
            first_depth: Some(0),
            next_depth: Some(0),
            ..Default::default()
        };

        // Test path: `a/b/x/y`.
        let mut search_engine = HashMap::new();
        search_engine.insert("a".into(), vec!["a/b".into()]);
        search_engine.insert("a/b".into(), vec!["a/b/x".into()]);
        search_engine.insert("a/b/x".into(), vec!["a/b/x/y".into()]);

        // path: [a]
        // slices: [a]/b ...
        let sequence_ab: Sequence = Sequence::from_str("a/b").unwrap();
        let sequence_xy: Sequence = Sequence::from_str("x/y").unwrap();
        let entry_a =
            Entry::new("a".into(), vec![sequence_ab, sequence_xy], opts);
        let result = entry_a.fire_walk(&search_engine).unwrap();

        // path: a/[b]
        // slices: a/[b] ...
        let entry_ab =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_ab.fire_walk(&search_engine).unwrap();

        // path: a/b/[x]
        // slices: ... [x]/y
        let entry_abx =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_abx.fire_walk(&search_engine).unwrap();

        // path: a/b/x/y
        // slices: ... x/[y]
        let entry_abxy =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_abxy.fire_walk(&search_engine).unwrap();

        variant!(result, FullMatch(_, _));
    }

    #[test]
    fn test_next_depth_exceeded() {
        let opts = SearchOpts {
            first_depth: Some(0),
            next_depth: Some(0),
            ..Default::default()
        };

        // Test path: `a/b/o/x/y`.
        let mut search_engine = HashMap::new();
        search_engine.insert("a".into(), vec!["a/b".into()]);
        search_engine.insert("a/b".into(), vec!["a/b/o".into()]);
        search_engine.insert("a/b/o".into(), vec!["a/b/o/x".into()]);
        search_engine.insert("a/b/o/x".into(), vec!["a/b/o/x/y".into()]);

        // path: [a]
        // slices: [a]/b ...
        let sequence_ab: Sequence = Sequence::from_str("a/b").unwrap();
        let sequence_xy: Sequence = Sequence::from_str("x/y").unwrap();
        let entry_a =
            Entry::new("a".into(), vec![sequence_ab, sequence_xy], opts);
        let result = entry_a.fire_walk(&search_engine).unwrap();

        // path: a/[b]
        // slices: a/[b] ...
        let entry_ab =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_ab.fire_walk(&search_engine).unwrap();

        // path: a/b/[o]
        // slices: ... [x]/y
        let entry_abx =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_abx.fire_walk(&search_engine).unwrap();

        variant!(result, DeadEnd);
    }

    #[test]
    fn test_next_depth() {
        use MatchStrength::*;

        let opts = SearchOpts {
            first_depth: Some(0),
            next_depth: Some(1),
            ..Default::default()
        };

        // Test path: `a/b/o/x/y`.
        let mut search_engine = HashMap::new();
        search_engine.insert("a".into(), vec!["a/b".into()]);
        search_engine.insert("a/b".into(), vec!["a/b/o".into()]);
        search_engine.insert("a/b/o".into(), vec!["a/b/o/x".into()]);
        search_engine.insert("a/b/o/x".into(), vec!["a/b/o/x/y".into()]);

        // path: [a]
        // slices: [a]/b ...
        let sequence_ab: Sequence = Sequence::from_str("a/b").unwrap();
        let sequence_xy: Sequence = Sequence::from_str("x/y").unwrap();
        let entry_a =
            Entry::new("a".into(), vec![sequence_ab, sequence_xy], opts);
        let result = entry_a.fire_walk(&search_engine).unwrap();

        // path: a/[b]
        // slices: a/[b] ...
        let entry_ab =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_ab.fire_walk(&search_engine).unwrap();

        // path: a/b/[o]
        // slices: ... [x]/y
        let entry_abo =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_abo.fire_walk(&search_engine).unwrap();

        // path: a/b/o/[x]
        // slices: ... [x]/y
        let entry_abox = variant!(result, Advancement(children, Naught) => children[0].clone());
        let result = entry_abox.fire_walk(&search_engine).unwrap();

        // path: a/b/o/x/[y]
        // slices: ... x/[y]
        let entry_aboxy =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_aboxy.fire_walk(&search_engine).unwrap();

        variant!(result, FullMatch(_, _));
    }

    #[test]
    fn test_next_depth_premature_match() {
        use MatchStrength::*;

        let opts = SearchOpts {
            first_depth: Some(0),
            next_depth: Some(1),
            ..Default::default()
        };

        // Test path: `a/b/o/x/o`.
        let mut search_engine = HashMap::new();
        search_engine.insert("a".into(), vec!["a/b".into()]);
        search_engine.insert("a/b".into(), vec!["a/b/o".into()]);
        search_engine.insert("a/b/o".into(), vec!["a/b/o/x".into()]);
        search_engine.insert("a/b/o/x".into(), vec!["a/b/o/x/o".into()]);

        // path: [a]
        // slices: [a]/b ...
        let sequence_ab: Sequence = Sequence::from_str("a/b").unwrap();
        let sequence_xy: Sequence = Sequence::from_str("x/y").unwrap();
        let entry_a =
            Entry::new("a".into(), vec![sequence_ab, sequence_xy], opts);
        let result = entry_a.fire_walk(&search_engine).unwrap();

        // path: a/[b]
        // slices: a/[b] ...
        let entry_ab =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_ab.fire_walk(&search_engine).unwrap();

        // path: a/b/[o]
        // slices: ... [x]/y
        let entry_abo =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_abo.fire_walk(&search_engine).unwrap();

        // path: a/b/o/[x]
        // slices: ... [x]/y
        let entry_abox = variant!(result, Advancement(children, Naught) => children[0].clone());
        let result = entry_abox.fire_walk(&search_engine).unwrap();

        // path: a/b/o/x/[o]
        // slices: ... x/[y]
        let entry_aboxy =
            variant!(result, Advancement(children, _) => children[0].clone());
        let result = entry_aboxy.fire_walk(&search_engine).unwrap();

        variant!(result, DeadEnd);
    }
}
