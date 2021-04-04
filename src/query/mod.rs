#[allow(dead_code)]
use crate::{Error, Result};

use std::path::PathBuf;

use clap::ArgMatches;
use regex::Regex;

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

struct Entry {
    slices: Vec<Sequence>,
    path: PathBuf,
}

struct SearchOpts {
    first_depth: Option<usize>,
    next_depth: Option<usize>,
    start_dir: PathBuf,
}

#[derive(Clone)]
struct Slice(String);

impl Slice {
    fn match_component(&self, component: &str) -> SliceMatch {
        if component.contains(&self.0) {
            SliceMatch::Yes(self.0 == component)
        } else {
            SliceMatch::No
        }
    }
}

#[derive(Clone)]
struct Sequence {
    slice_to_match: usize,
    slices: Vec<Slice>,
}

enum SliceMatch {
    Yes(bool),
    No,
}

enum SequenceFlow {
    DeadEnd,
    Continue(Sequence, bool),
    Next(bool),
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

    fn match_component(&self, component: &str) -> SequenceFlow {
        let (slice, is_last) = match self.slices.get(self.slice_to_match..) {
            // Invalid `Sequence` constructed. TODO: Log.
            None => return SequenceFlow::DeadEnd,
            // Invalid `Sequence` constructed. TODO: Log.
            Some([]) => return SequenceFlow::DeadEnd,
            Some([slice]) => (slice, true),
            Some([slice, _, ..]) => (slice, false),
        };
        let is_first = (self.slice_to_match == 0);

        match slice.match_component(component) {
            SliceMatch::Yes(full_match) =>
                if is_last {
                    SequenceFlow::Next(full_match)
                } else {
                    let sequence = Sequence {
                        slice_to_match: self.slice_to_match + 1,
                        slices: self.slices.clone(),
                    };

                    SequenceFlow::Continue(sequence, full_match)
                },
            SliceMatch::No =>
                if is_first {
                    SequenceFlow::Continue(self.clone(), false)
                } else {
                    let sequence = Sequence {
                        slice_to_match: 0,
                        slices: self.slices.clone(),
                    };

                    SequenceFlow::Continue(sequence, false)
                },
        }
    }
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
    fn test_abc() {
        use SequenceFlow::*;

        let sequence_abc: Sequence = Sequence::from_str(r"a/b/c").unwrap();

        // Test path: `a/abba/candy`.

        let sequence_bc = match sequence_abc.match_component("a") {
            Continue(sequence, true) => sequence,
            _ => panic!(),
        };

        let sequence_c = match sequence_bc.match_component("abba") {
            Continue(sequence, false) => sequence,
            _ => panic!(),
        };

        let () = match sequence_c.match_component("candy") {
            Next(false) => {}
            _ => panic!(),
        };
    }

    #[test]
    fn test_xy() {
        use SequenceFlow::*;

        let sequence_xy: Sequence = Sequence::from_str("x/y").unwrap();

        // Test path: `x/o/ox/ymoron`.

        let sequence_y = match sequence_xy.match_component("x") {
            Continue(sequence, true) => sequence,
            _ => panic!(),
        };

        let sequence_xy = match sequence_y.match_component("o") {
            Continue(sequence, false) => sequence,
            _ => panic!(),
        };

        let sequence_y = match sequence_xy.match_component("ox") {
            Continue(sequence, false) => sequence,
            _ => panic!(),
        };

        let () = match sequence_y.match_component("ymoron") {
            Next(false) => {}
            _ => panic!(),
        };
    }
}
