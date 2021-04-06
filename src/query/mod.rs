#[allow(dead_code)]
use crate::{Error, Result};

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use clap::ArgMatches;

mod search_engine;
mod sequence;
mod slice;

use search_engine::SearchEngine;
use sequence::{Sequence, SequenceFlow};

pub fn query(matches: &ArgMatches<'_>) -> Result<Vec<PathBuf>> {
    let (opts, slices) = parse_args(matches)?;

    if slices.is_empty() {
        return Ok(vec![opts.start_dir]);
    }

    search(opts, slices)
}

fn search(_opts: SearchOpts, _slices: Vec<Sequence>) -> Result<Vec<PathBuf>> {
    Err(dev_err!("unimplemented"))
}

#[derive(Debug, Clone)]
pub struct Entry {
    sequences: Vec<Sequence>,
    path: PathBuf,
    opts: SearchOpts,
    attempt_count: usize,
    last_match: Option<usize>,
}

impl Entry {
    pub fn new(
        path: PathBuf,
        sequences: Vec<Sequence>,
        opts: SearchOpts,
    ) -> Self {
        Self {
            path,
            sequences,
            opts,
            attempt_count: 0,
            last_match: None,
        }
    }

    // TODO: Rename this.
    pub fn fire_walk<E>(&self, engine: E) -> Result<EntryMatch>
    where
        E: SearchEngine,
    {
        use EntryMatch::*;

        let component = self
            .path
            .file_name()
            .ok_or(dev_err!("no filename in entry path"))?
            .to_string_lossy();

        let sequence = self
            .sequences
            .get(0)
            .ok_or(dev_err!("invalid current sequence index"))?;

        let sequence_match = sequence.match_component(
            &component,
            self.attempt_count,
            self.last_match,
            &self.opts,
        )?;

        let result = match sequence_match {
            SequenceFlow::Next(strength) => {
                let is_last = (self.sequences.len() <= 1);

                if is_last {
                    FullMatch(self.path.clone(), strength)
                } else {
                    let sequences = self
                        .sequences
                        .get(1..)
                        .ok_or(dev_err!("entry with no sequences"))?;

                    let children =
                        self.get_children(sequences.to_vec(), engine);

                    Advancement(children, strength)
                }
            }
            SequenceFlow::Continue(sequence, strength) => {
                let mut sequences = self.sequences.clone();
                let first_sequence = sequences
                    .get_mut(0)
                    .ok_or(dev_err!("entry with no sequences"))?;
                *first_sequence = sequence;

                let children = self.get_children(sequences, engine);

                Advancement(children, strength)
            }
            SequenceFlow::DeadEnd => DeadEnd,
        };

        Ok(result)
    }

    fn get_children<E>(&self, sequences: Vec<Sequence>, engine: E) -> Vec<Entry>
    where
        E: SearchEngine,
    {
        engine
            .read_dir(&self.path)
            .into_iter()
            .map(|child_path| Entry {
                attempt_count: self.attempt_count + 1,
                sequences: sequences.clone(),
                path: child_path,
                opts: self.opts.clone(),
                last_match: None,
            })
            .collect()
    }
}

#[derive(Debug)]
pub enum EntryMatch {
    Advancement(Vec<Entry>, MatchStrength),
    FullMatch(PathBuf, MatchStrength),
    DeadEnd,
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
    let slices = matches
        .values_of("SLICES")
        .ok_or(dev_err!("required `clap` arg absent"))?;
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
        start_dir: PathBuf::from("."),
    };

    Ok((opts, slices))
}

#[cfg(test)]
mod test {
    use super::*;

    fn as_path<P>(path: &P) -> &Path
    where
        P: AsRef<Path> + ?Sized,
    {
        path.as_ref()
    }

    #[test]
    fn test_sequence_from_str() {
        assert!(Sequence::from_str(r"").is_err());
        assert!(Sequence::from_str(r".").is_err());
        assert!(Sequence::from_str(r"ab cd").is_err());
        assert!(Sequence::from_str(r"\").is_err());
        assert!(Sequence::from_str(r"\").is_err());

        // assert!(Sequence::from_str(r"zażółć").is_ok());
        assert!(Sequence::from_str(r"ab/cd/ef").is_ok());
        assert!(Sequence::from_str(r"abc").is_ok());
    }

    #[test]
    fn test_basic_match() {
        use MatchStrength::*;
        use SequenceFlow::*;

        let sequence_abc: Sequence = Sequence::from_str(r"a/b/c").unwrap();
        let opts = SearchOpts::default();
        let last_match = None;

        // Test path: `a/bee/ice`.

        let result = sequence_abc
            .match_component("a", 0, last_match, &opts)
            .unwrap();
        let sequence_bc = variant!(result, Continue(sequence, Complete) => sequence);

        let result = sequence_bc
            .match_component("bee", 1, last_match, &opts)
            .unwrap();
        let sequence_c = variant!(result, Continue(sequence, Partial) => sequence);

        let result = sequence_c
            .match_component("ice", 2, last_match, &opts)
            .unwrap();
        variant!(result, Next(Partial) => ());
    }

    #[test]
    fn test_recover_premature_match() {
        use MatchStrength::*;
        use SequenceFlow::*;

        let sequence_xy: Sequence = Sequence::from_str("x/y").unwrap();
        let opts = SearchOpts::default();
        let last_match = None;

        // Test path: `x/o/ox/ymoron`.

        let result = sequence_xy
            .match_component("x", 0, last_match, &opts)
            .unwrap();
        let sequence_y = variant!(result, Continue(sequence, Complete) => sequence);

        let result = sequence_y
            .match_component("o", 1, last_match, &opts)
            .unwrap();
        let sequence_xy = variant!(result, Continue(sequence, Naught) => sequence);

        let result = sequence_xy
            .match_component("ox", 2, last_match, &opts)
            .unwrap();
        let sequence_y = variant!(result, Continue(sequence, Partial) => sequence);

        let result = sequence_y
            .match_component("ymoron", 3, last_match, &opts)
            .unwrap();
        variant!(result, Next(Partial) => ());
    }

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
        assert_eq!(entry_a.path, as_path("a"));
        let result = entry_a.fire_walk(&search_engine).unwrap();

        // path: a/[o]
        // slices: a/[b]
        let entry_ao = variant!(result, Advancement(children, Complete) => children[0].clone());
        assert_eq!(entry_ao.path, as_path("a/o"));
        let result = entry_ao.fire_walk(&search_engine).unwrap();

        // path: a/o/[a]
        // slices: [a]/b
        let entry_aoa = variant!(result, Advancement(children, Naught) => children[0].clone());
        assert_eq!(entry_aoa.path, as_path("a/o/a"));
        let result = entry_aoa.fire_walk(&search_engine).unwrap();

        // path: a/o/a/[b]
        // slices: a/[b]
        let entry_aoab = variant!(result, Advancement(children, Complete) => children[0].clone());
        assert_eq!(entry_aoab.path, as_path("a/o/a/b"));
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
}
