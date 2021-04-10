use crate::{Error, Result};


use super::{
    abbr::{Abbr, Congruence},
    search_engine::SearchEngine,
};


use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};


#[derive(Debug, Clone)]
pub struct Entry<'a> {
    abbr: &'a Abbr,
    rest: &'a [Abbr],
    pub(super) path: PathBuf,
    congruence: Vec<Congruence>,
}


impl<'a> Entry<'a> {
    pub fn new(
        path: PathBuf,
        abbr: &'a Abbr,
        rest: &'a [Abbr],
    ) -> Result<Self> {
        // Safety check. Return error on wildcard at last place.
        if let (Abbr::Wildcard, []) = (abbr, rest) {
            return Err(Error::WildcardAtLastPlace);
        }

        Ok(Self {
            path,
            abbr,
            rest,
            congruence: vec![],
        })
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn n_attempts(&self) -> usize {
        self.congruence.len()
    }

    pub fn congruence(&self) -> &[Congruence] {
        self.congruence.as_slice()
    }

    pub fn advance<E>(&self, engine: E) -> Flow<'a>
    where
        E: SearchEngine,
    {
        let component = self
            .path
            .file_name()
            .map(|file_name| file_name.to_string_lossy());
        let component: Cow<_> = match component {
            None => return Flow::DeadEnd,
            Some(component) => component,
        };


        let congruence = self.abbr.compare(&component);

        match congruence {
            Some(next_congruence) => match self.rest {
                [abbr, rest @ ..] => {
                    let mut congruence = self.congruence.clone();
                    congruence.push(next_congruence);

                    let children = Self::construct_children(
                        &self.path, abbr, rest, congruence, engine,
                    );

                    Flow::Continue(children)
                }
                [] => {
                    let mut congruence = self.congruence.clone();
                    congruence.push(next_congruence);

                    let entry = Entry {
                        congruence,
                        path: self.path.clone(),
                        ..*self
                    };

                    Flow::FullMatch(entry)
                }
            },
            None => Flow::DeadEnd,
        }
    }

    fn construct_children<E>(
        path: &Path,
        abbr: &'a Abbr,
        rest: &'a [Abbr],
        congruence: Vec<Congruence>,
        engine: E,
    ) -> Vec<Entry<'a>>
    where
        E: SearchEngine,
    {
        engine
            .read_dir(path)
            .iter()
            .map(|child_path| Entry {
                path: child_path.into(),
                congruence: congruence.clone(),
                abbr,
                rest,
            })
            .collect()
    }
}


#[derive(Debug)]
pub enum Flow<'a> {
    Continue(Vec<Entry<'a>>),
    FullMatch(Entry<'a>),
    DeadEnd,
}
