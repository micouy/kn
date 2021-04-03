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
pub struct EntryNode<'a>(pub PathBuf, pub &'a [PathSlice]);

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
                return DeadEnd;
            }
        };
        let comp: &str = &comp;

        use DigResult::*;
        let whole = self.1;

        match whole {
            [] => FullMatch,
            [PathSlice::Glued(re), rest @ ..] =>
                if re.is_match(comp) {
                    match rest {
                        [_, ..] => Continue(self.prepare_children(rest)),
                        [] => FullMatch,
                    }
                } else {
                    DeadEnd
                },
            [PathSlice::Loose(re), rest @ ..] =>
                if re.is_match(comp) {
                    match rest {
                        [_, ..] => {
                            // Continue
                            let children =
                                self.prepare_children(rest).collect::<Vec<_>>();
                            if self.0.starts_with("/Users/mikolaj/mine") {
                                log::debug!("node: {}", self.0.display());
                                log::debug!("children: {:?}", children);
                            }
                            Continue(box children.into_iter())
                        }
                        [] => FullMatch,
                    }
                } else {
                    Continue(self.prepare_children(whole))
                },
        }
    }

    fn prepare_children(
        &self,
        slices_left: &'a [PathSlice],
    ) -> Box<dyn Iterator<Item = EntryNode<'a>> + 'a> {
        log::trace!("prepare children");

        let read_dir = match self.0.read_dir() {
            Ok(read_dir) => read_dir,
            Err(err) => {
                log::warn!("cant read dir: {}", self.0.display());
                log::warn!("error message: {}", err);

                return box std::iter::empty();
            }
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
