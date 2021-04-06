use super::{
    slice::{Slice, SliceMatch},
    MatchStrength,
    SearchOpts,
};
use crate::{Error, Result};
use regex::Regex;

#[derive(Clone, Debug)]
pub struct Sequence {
    slice_to_match: usize,
    slices: Vec<Slice>,
}

impl Sequence {
    pub fn from_str(slices: &str) -> Result<Self> {
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

    pub fn match_component(
        &self,
        component: &str,
        attempt: usize,
        last_match: Option<usize>,
        opts: &SearchOpts,
    ) -> Result<SequenceFlow> {
        use MatchStrength::*;

        let (slice, is_last) = match self.slices.get(self.slice_to_match..) {
            None => return Err(dev_err!("empty sequence constructed")),
            Some([]) => return Err(dev_err!("empty sequence constructed")),
            Some([slice]) => (slice, true),
            Some([slice, _, ..]) => (slice, false),
        };

        let result = match slice.match_component(component) {
            SliceMatch::Yes(strength) =>
            // No need to check opts. If the sequence was properly
            // constructed and matched, the options are not violated.
                if is_last {
                    SequenceFlow::Next(strength)
                } else {
                    let sequence = Sequence {
                        slice_to_match: self.slice_to_match + 1,
                        slices: self.slices.clone(),
                    };

                    SequenceFlow::Continue(sequence, strength)
                },
            SliceMatch::No => {
                // Firstly, check if there's any chance for child entries
                // to match. If not, return `DeadEnd`.

                match last_match {
                    Some(last_match) =>
                        if let Some(next_depth) = opts.next_depth {
                            if (last_match + next_depth + 1) <= attempt {
                                return Ok(SequenceFlow::DeadEnd);
                            }
                        },
                    None =>
                        if let Some(first_depth) = opts.first_depth {
                            if first_depth <= attempt {
                                return Ok(SequenceFlow::DeadEnd);
                            }
                        },
                }

                let sequence = Sequence {
                    slice_to_match: 0,
                    slices: self.slices.clone(),
                };

                SequenceFlow::Continue(sequence, Naught)
            }
        };

        Ok(result)
    }
}

#[derive(Clone, Debug)]
pub enum SequenceFlow {
    Continue(Sequence, MatchStrength),
    /// Move on to the next sequence.
    Next(MatchStrength),
    DeadEnd,
}
