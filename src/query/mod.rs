#[allow(dead_code)]
use crate::{Error, Result};

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use clap::ArgMatches;
use regex::Regex;

mod search_engine;

use search_engine::SearchEngine;

macro_rules! variant {
    ($expression_in:expr, $pattern:pat => $expression_out:expr) => {
        match $expression_in {
            $pattern => $expression_out,
            _ => panic!(),
        }
    };

    ($expression_in:expr, $pattern:pat => $expression_out:expr, $panic:expr) => {
        match $expression_in {
            $pattern => $expression_out,
            _ => panic!($panic),
        }
    };
}

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
struct Entry {
    sequences: Vec<Sequence>,
    path: PathBuf,
    opts: SearchOpts,
    attempt_count: usize,
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
        }
    }

    // TODO: Rename this.
    fn fire_walk<E>(&self, engine: E) -> Result<EntryMatch>
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

        let attempt_count = self.attempt_count + 1;

        // TODO: Use info about whether it was a complete match or not.
        let result = match sequence.match_component(
            &component,
            self.attempt_count,
            &self.opts,
        )? {
            SequenceMatch::Next(strength) => {
                let is_last = (self.sequences.len() <= 1);

                if is_last {
                    FullMatch(self.path.clone(), strength)
                } else {
                    let sequences = self
                        .sequences
                        .get(1..)
                        .ok_or(dev_err!("entry with no sequences"))?;

                    let children = engine
                        .read_dir(&self.path)
                        .into_iter()
                        .map(|child_path| Entry {
                            attempt_count,
                            path: child_path,
                            sequences: sequences.to_vec(),
                            opts: self.opts.clone(),
                        })
                        .collect();

                    Continue(children, strength)
                }
            }
            SequenceMatch::Current(new_sequence, strength) => {
                let mut sequences = self.sequences.clone();
                let first_sequence = sequences
                    .get_mut(0)
                    .ok_or(dev_err!("entry with no sequences"))?;
                *first_sequence = new_sequence;

                let children = engine
                    .read_dir(&self.path)
                    .into_iter()
                    .map(|child_path| Entry {
                        attempt_count,
                        sequences: sequences.clone(),
                        path: child_path,
                        opts: self.opts.clone(),
                    })
                    .collect();

                Continue(children, strength)
            }
        };

        Ok(result)
    }
}

#[derive(Debug)]
enum EntryMatch {
    Continue(Vec<Entry>, MatchStrength),
    FullMatch(PathBuf, MatchStrength),
    DeadEnd,
}

#[derive(Debug, Clone, Default)]
struct SearchOpts {
    first_depth: Option<usize>,
    next_depth: Option<usize>,
    start_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct Slice(String);

impl Slice {
    fn match_component(&self, component: &str) -> SliceMatch {
        use MatchStrength::*;

        if component.contains(&self.0) {
            if self.0 == component {
                SliceMatch::Yes(Complete)
            } else {
                SliceMatch::Yes(Partial)
            }
        } else {
            SliceMatch::No
        }
    }
}

// TODO: Think of better names for variants.
#[derive(Clone, Debug)]
enum SliceMatch {
    Yes(MatchStrength),
    No,
}

#[derive(Clone, Debug)]
enum MatchStrength {
    Complete,
    Partial,
}

#[derive(Clone, Debug)]
struct Sequence {
    slice_to_match: usize,
    slices: Vec<Slice>,
}

impl Sequence {
    fn from_str(slices: &str) -> Result<Self> {
        let only_valid_re = Regex::new(r"^[\-_.a-zA-Z0-9]+$").unwrap();
        let only_dots_re = Regex::new(r"^\.+$").unwrap();

        let is_valid = |slice: &str| {
            if slice.is_empty() {
                return false;
            }
            if !only_valid_re.is_match(slice) {
                return false;
            };
            if only_dots_re.is_match(slice) {
                return false;
            };

            return true;
        };

        let slices = slices
            .split("/")
            .map(|slice| {
                if !is_valid(slice) {
                    Err(Error::InvalidSliceSequence(slices.to_string()))
                } else {
                    Ok(Slice(slice.to_string()))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            slices,
            slice_to_match: 0,
        })
    }

    fn match_component(
        &self,
        component: &str,
        attempt: usize,
        opts: &SearchOpts,
    ) -> Result<SequenceMatch> {
        use MatchStrength::*;

        let (slice, is_last) = match self.slices.get(self.slice_to_match..) {
            None => return Err(dev_err!("empty sequence constructed")),
            Some([]) => return Err(dev_err!("empty sequence constructed")),
            Some([slice]) => (slice, true),
            Some([slice, _, ..]) => (slice, false),
        };
        let is_first = (self.slice_to_match == 0);

        let result = match slice.match_component(component) {
            SliceMatch::Yes(strength) =>
                if is_last {
                    SequenceMatch::Next(strength)
                } else {
                    let sequence = Sequence {
                        slice_to_match: self.slice_to_match + 1,
                        slices: self.slices.clone(),
                    };

                    SequenceMatch::Current(sequence, strength)
                },
            SliceMatch::No =>
            // TODO: Change `Partial` in this case to `None`?
            // This will be needed to handle max match depth.
                if is_first {
                    SequenceMatch::Current(self.clone(), Partial)
                } else {
                    let sequence = Sequence {
                        slice_to_match: 0,
                        slices: self.slices.clone(),
                    };

                    SequenceMatch::Current(sequence, Partial)
                },
        };

        Ok(result)
    }
}

#[derive(Clone, Debug)]
enum SequenceMatch {
    Current(Sequence, MatchStrength),
    Next(MatchStrength),
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
        use SequenceMatch::*;

        let sequence_abc: Sequence = Sequence::from_str(r"a/b/c").unwrap();
        let opts = SearchOpts::default();

        // Test path: `a/abba/candy`.

        let sequence_bc =
            match sequence_abc.match_component("a", 0, &opts).unwrap() {
                Current(sequence, Complete) => sequence,
                other => panic!("{:?}", other),
            };

        let sequence_c =
            match sequence_bc.match_component("abba", 1, &opts).unwrap() {
                Current(sequence, Partial) => sequence,
                other => panic!("{:?}", other),
            };

        let () = match sequence_c.match_component("candy", 2, &opts).unwrap() {
            Next(Partial) => {}
            other => panic!("{:?}", other),
        };
    }

    #[test]
    fn test_recover_premature_match() {
        use MatchStrength::*;
        use SequenceMatch::*;

        let sequence_xy: Sequence = Sequence::from_str("x/y").unwrap();
        let opts = SearchOpts::default();

        // Test path: `x/o/ox/ymoron`.

        let sequence_y =
            match sequence_xy.match_component("x", 0, &opts).unwrap() {
                Current(sequence, Complete) => sequence,
                other => panic!("{:?}", other),
            };

        let sequence_xy =
            match sequence_y.match_component("o", 1, &opts).unwrap() {
                Current(sequence, Partial) => sequence,
                other => panic!("{:?}", other),
            };

        let sequence_y =
            match sequence_xy.match_component("ox", 2, &opts).unwrap() {
                Current(sequence, Partial) => sequence,
                other => panic!("{:?}", other),
            };

        let () = match sequence_y.match_component("ymoron", 3, &opts).unwrap() {
            Next(Partial) => {}
            other => panic!("{:?}", other),
        };
    }

    #[test]
    fn test_entry_walk() {
        use EntryMatch::*;
        use MatchStrength::*;
        use SequenceMatch::*;

        let sequence_ab: Sequence = Sequence::from_str("a/b").unwrap();
        let opts = SearchOpts {
            first_depth: Some(0),
            ..Default::default()
        };
        let entry_a = Entry {
            sequences: vec![sequence_ab],
            opts,
            path: "a".into(),
            attempt_count: 0,
        };

        // Test path: `a/o/a/b`.
        let mut search_engine = HashMap::new();
        search_engine.insert("a".into(), vec!["a/o".into()]);
        search_engine.insert("a/o".into(), vec!["a/o/a".into()]);
        search_engine.insert("a/o/a".into(), vec!["a/o/a/b".into()]);

        let sequence_ab = &entry_a.sequences[0];

        // The square brackets indicate which component will be matched against
        // which slice.

        // path: [a]
        // slices: [a]/b
        assert_eq!(entry_a.path, as_path("a"));
        assert_eq!(sequence_ab.slices[sequence_ab.slice_to_match].0, "a");

        let result = entry_a.fire_walk(&search_engine).unwrap();
        let entry_ao = variant!(result, Continue(children, Complete) => children[0].clone());
        let sequence_b = &entry_ao.sequences[0];
        let current_path = &entry_ao.path;
        let current_slice = &sequence_b.slices[sequence_b.slice_to_match].0;

        // path: a/[o]
        // slices: a/[b]
        assert_eq!(current_path, as_path("a/o"));
        assert_eq!(current_slice, "b");

        let result = entry_ao.fire_walk(&search_engine).unwrap();
        // TODO: Change `Partial` to `None` here?
        let entry_aoa = variant!(result, Continue(children, Partial) => children[0].clone());
        let sequence_ab = &entry_aoa.sequences[0];
        let current_path = &entry_aoa.path;
        let current_slice = &sequence_ab.slices[sequence_ab.slice_to_match].0;

        // path: a/o/[a]
        // slices: [a]/b
        assert_eq!(current_path, as_path("a/o/a"));
        assert_eq!(current_slice, "a");

        let result = entry_aoa.fire_walk(&search_engine).unwrap();
        let entry_aoab = variant!(result, Continue(children, Complete) => children[0].clone());
        let sequence_b = &entry_aoab.sequences[0];
        let current_path = &entry_aoab.path;
        let current_slice = &sequence_b.slices[sequence_b.slice_to_match].0;

        // path: a/o/a/[b]
        // slices: a/[b]
        assert_eq!(current_path, as_path("a/o/a/b"));
        assert_eq!(current_slice, "b");

        let result = entry_aoab.fire_walk(&search_engine).unwrap();
        let path = variant!(result, FullMatch(path, Complete) => path);

        assert_eq!(path, as_path("a/o/a/b"));
    }
}
