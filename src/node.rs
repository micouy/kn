//! Things related to [`PathSlice`](crate::node::PathSlice)s and
//! [`EntryNode`](crate::node::EntryNode)s.

use std::path::PathBuf;

use regex::Regex;

/// A slice of path.
#[derive(Debug)]
pub enum PathSlice {
    /// A slice of path that must be matched right after the previous one.
    Glued(Regex),
    /// A slice of path that can be matched a number of components after the
    /// previous one.
    Loose(Regex),
}

/// A result of matching a component against slices left.
pub enum MatchResult<'a> {
    FullMatch,
    Continue(PathSlices<'a>),
}

/// A wrapper around [`&[PathSlice]`](crate::node::PathSlice). Allows to easily
/// recover from a premature match of a glued slice.
#[derive(Copy, Clone, Debug)]
pub struct PathSlices<'a> {
    last_match: Option<usize>,
    slices: &'a [PathSlice],
}

impl<'a> PathSlices<'a> {
    pub fn new(slices: &'a [PathSlice]) -> Self {
        Self {
            last_match: None,
            slices,
        }
    }

    pub fn try_match(&self, comp: &str) -> MatchResult<'a> {
        use MatchResult::*;

        let ix = self
            .last_match
            .map(|last_match| last_match + 1)
            .unwrap_or(0);

        match self.slices.get(ix) {
            None => return FullMatch, // Full match? Or error?
            Some(slice) => match slice {
                PathSlice::Loose(re) =>
                    if re.is_match(comp) {
                        let last_match = Some(ix);
                        let is_match_last = (ix >= self.slices.len() - 1);

                        if is_match_last {
                            return FullMatch;
                        } else {
                            return Continue(PathSlices {
                                last_match,
                                slices: self.slices,
                            });
                        }
                    } else {
                        return Continue(*self);
                    },
                PathSlice::Glued(re) =>
                    if re.is_match(comp) {
                        let last_match = Some(ix);
                        let is_match_last = (ix >= self.slices.len() - 1);

                        if is_match_last {
                            return FullMatch;
                        } else {
                            return Continue(PathSlices {
                                last_match,
                                slices: self.slices,
                            });
                        }
                    } else {
                        // return to the last loose slice
                        let last_match = (0..ix)
                            .rev()
                            .filter_map(|i| {
                                if let Some(PathSlice::Loose(_)) =
                                    self.slices.get(i)
                                {
                                    if i == 0 {
                                        None
                                    } else {
                                        Some(i - 1)
                                    }
                                } else {
                                    None
                                }
                            })
                            .next();

                        return Continue(PathSlices {
                            last_match,
                            slices: self.slices,
                        });
                    },
            },
        }
    }
}

/// A result of digging one level further down the file tree.
pub enum DigResult<'a> {
    /// Add path to fully matched paths.
    FullMatch,
    /// End search down that path.
    DeadEnd,
    /// Continue search. Contains all possible paths of traversal from the
    /// node.
    Continue(Box<dyn Iterator<Item = EntryNode<'a>> + 'a>),
}

/// A container for an entry and args left to match.
#[derive(Debug)]
pub struct EntryNode<'a>(pub PathBuf, pub PathSlices<'a>);

impl<'a> EntryNode<'a> {
    /// Dig one level further down the file tree.
    pub fn dig_deeper<'c>(&self) -> DigResult<'a> {
        log::trace!("dig deeper");

        // TODO: Clean up this paragraph.
        let comp: Option<String> = self
            .0
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        let comp = match comp {
            Some(comp) => comp,
            None => {
                // TODO: Fix later. Probably not a problem
                // since all valid entries should have a filename.
                return DigResult::DeadEnd;
            }
        };
        let comp: &str = &comp;

        match self.1.try_match(comp) {
            MatchResult::FullMatch => DigResult::FullMatch,
            MatchResult::Continue(slices) =>
                DigResult::Continue(self.prepare_children(slices)),
        }
    }

    fn prepare_children(
        &self,
        slices_left: PathSlices<'a>,
    ) -> Box<dyn Iterator<Item = EntryNode<'a>> + 'a> {
        log::trace!("prepare children");

        let read_dir = match self.0.read_dir() {
            Ok(read_dir) => read_dir,
            Err(_) => return box std::iter::empty(),
        };

        let children = read_dir
            .filter_map(|res| res.ok())
            .filter(|entry| {
                entry.file_type().map(|meta| meta.is_dir()).unwrap_or(false)
            })
            .map(move |child_entry| EntryNode(child_entry.path(), slices_left));

        box children
    }
}
