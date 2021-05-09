use crate::{Error, Result};

use super::{
    abbr::{Abbr, Congruence},
    fs::FileSystem,
};

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Entry {
    pub path: PathBuf,
    pub congruence: Vec<Congruence>,
}

/*
struct Finding {
    dupa: Vec<(Entry, Congruence)>,
    niech entry nawet nie zawiera congruence
}

fn advance(&self, abbr) -> Congruence

if let Some(congruence) = entry.advance() { // to powinno być entry.check czy coś
    finding.push((Entry, congruence));
}
*/

impl Entry {
    pub fn new(path: PathBuf, congruence: Vec<Congruence>) -> Self {
        Self { path, congruence }
    }

    pub fn advance(&self, abbr: &Abbr) -> Flow {
        let component = self
            .path
            .file_name()
            .map(|file_name| file_name.to_string_lossy());
        let component: Cow<_> = match component {
            None => return Flow::DeadEnd,
            Some(component) => component,
        };

        let congruence = abbr.compare(&component);

        match congruence {
            Some(next_congruence) => {
                let mut congruence = self.congruence.clone();
                congruence.push(next_congruence);

                let entry = Entry {
                    path: self.path.clone(),
                    congruence,
                };

                Flow::Continue(entry)
            }
            None => Flow::DeadEnd,
        }
    }
}

#[derive(Debug)]
pub enum Flow {
    Continue(Entry),
    DeadEnd,
}
