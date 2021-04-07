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
        let sequence_bc =
            variant!(result, Continue(sequence, Complete) => sequence);

        let result = sequence_bc
            .match_component("bee", 1, last_match, &opts)
            .unwrap();
        let sequence_c =
            variant!(result, Continue(sequence, Partial) => sequence);

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
        let sequence_y =
            variant!(result, Continue(sequence, Complete) => sequence);

        let result = sequence_y
            .match_component("o", 1, last_match, &opts)
            .unwrap();
        let sequence_xy =
            variant!(result, Continue(sequence, Naught) => sequence);

        let result = sequence_xy
            .match_component("ox", 2, last_match, &opts)
            .unwrap();
        let sequence_y =
            variant!(result, Continue(sequence, Partial) => sequence);

        let result = sequence_y
            .match_component("ymoron", 3, last_match, &opts)
            .unwrap();
        variant!(result, Next(Partial) => ());
    }
}
