use super::{
    search_engine::SearchEngine,
    sequence::{Sequence, SequenceFlow},
    MatchStrength,
    SearchOpts,
};
use crate::{Error, Result};

use std::path::{Path, PathBuf};

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

    pub fn path(&self) -> &Path {
        self.path.as_path()
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
