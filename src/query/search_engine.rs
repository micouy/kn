use std::{
    collections::HashMap,
    convert::AsRef,
    path::{Path, PathBuf},
};

pub trait SearchEngine {
    fn read_dir<P>(&self, dir: P) -> Vec<PathBuf>
    where
        P: AsRef<Path>;

    fn is_dir<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>;
}

pub struct ReadDirEngine;

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
                    .filter(|entry| {
                        entry
                            .metadata()
                            .map(|meta| meta.is_dir())
                            .unwrap_or(false)
                    })
                    .map(|entry| entry.path())
                    .collect()
            })
            .unwrap_or_else(|_| vec![])
    }

    fn is_dir<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        path.as_ref().is_dir()
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

    fn is_dir<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        self.contains_key(path.as_ref())
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

    fn is_dir<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        (*self).is_dir(path)
    }
}
