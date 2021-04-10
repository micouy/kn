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
    path: PathBuf,
    n_attempts: usize,
}


impl<'a> Entry<'a> {
    pub fn new(path: PathBuf, abbr: &'a Abbr, rest: &'a [Abbr]) -> Self {
        Self {
            path,
            abbr,
            rest,
            n_attempts: 0,
        }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn attempt(&self) -> usize {
        self.n_attempts
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
            Some(congruence) => match self.rest {
                [abbr, rest @ ..] => {
                    let children = Self::construct_children(
                        &self.path,
                        abbr,
                        rest,
                        self.n_attempts + 1,
                        engine,
                    );

                    Flow::Continue(children, congruence)
                }
                [] => Flow::FullMatch(self.path.clone(), congruence),
            },
            None => Flow::DeadEnd,
        }
    }

    fn construct_children<E>(
        path: &Path,
        abbr: &'a Abbr,
        rest: &'a [Abbr],
        n_attempts: usize,
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
                abbr,
                rest,
                n_attempts,
            })
            .collect()
    }
}


#[derive(Debug)]
pub enum Flow<'a> {
    Continue(Vec<Entry<'a>>, Congruence),
    FullMatch(PathBuf, Congruence),
    DeadEnd,
}
