use super::{
    slice::{Slice, SliceMatch},
    MatchStrength,
    SearchOpts,
};
use crate::{Error, Result};

#[cfg(feature = "logging")]
use log::debug;

#[derive(Clone, Debug)]
pub struct Sequence {
    slice_to_match: usize,
    slices: Vec<Slice>,
}

impl Sequence {
    pub fn from_str(slices: &str) -> Result<Self> {

        let slices = slices
            .split("/")
            .map(|pattern| Slice::from_string(pattern.to_string()))
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
                                #[cfg(feature = "logging")]
                                if let Slice::Literal(string) = slice {
                                    debug!("Dead end. {} vs {}. Already at allowed `next_depth`.", string, component);
                                }

                                return Ok(SequenceFlow::DeadEnd);
                            }
                        },
                    None =>
                        if let Some(first_depth) = opts.first_depth {
                            if first_depth <= attempt {
                                #[cfg(feature = "logging")]
                                if let Slice::Literal(string) = slice {
                                    debug!("Dead end. {} vs {}. Already at allowed `first_depth`.", string, component);
                                }

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
