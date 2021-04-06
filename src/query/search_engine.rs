use std::{
    collections::HashMap,
    convert::AsRef,
    path::{Path, PathBuf},
};

pub trait SearchEngine {
    fn read_dir<P>(&self, dir: P) -> Vec<PathBuf>
    where
        P: AsRef<Path>;
}

struct ReadDirEngine;

impl SearchEngine for ReadDirEngine {
    fn read_dir<P>(&self, dir: P) -> Vec<PathBuf>
    where
        P: AsRef<Path>,
    {
        dir.as_ref()
            .read_dir()
            .map(|read_dir| {
                read_dir
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.path())
                    .collect()
            })
            .unwrap_or_else(|_| vec![])
    }
}

impl SearchEngine for HashMap<PathBuf, Vec<PathBuf>> {
    fn read_dir<P>(&self, dir: P) -> Vec<PathBuf>
    where
        P: AsRef<Path>,
    {
        self.get(dir.as_ref())
            .map(|children| children.clone())
            .unwrap_or_else(|| vec![])
    }
}

impl<T> SearchEngine for &T
where
    T: SearchEngine,
{
    fn read_dir<P>(&self, dir: P) -> Vec<PathBuf>
    where
        P: AsRef<Path>,
    {
        (*self).read_dir(dir)
    }
}
