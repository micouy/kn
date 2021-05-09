use std::{
    collections::HashMap,
    convert::AsRef,
    path::{Path, PathBuf},
};

use crate::Result;

pub trait FileSystem {
    fn read_dir<P>(&self, dir: P) -> Result<Vec<PathBuf>>
    where
        P: AsRef<Path>;

    fn is_dir<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>;
}

pub struct DefaultFileSystem;

impl FileSystem for DefaultFileSystem {
    fn read_dir<P>(&self, dir: P) -> Result<Vec<PathBuf>>
    where
        P: AsRef<Path>,
    {
        let entries = dir
            .as_ref()
            .read_dir()?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_type()
                    .map(|file_type| file_type.is_dir())
                    .unwrap_or(false)
            })
            .map(|entry| entry.path())
            .collect();

        Ok(entries)
    }

    fn is_dir<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        path.as_ref().is_dir()
    }
}

pub type MockFileSystem = HashMap<PathBuf, Vec<PathBuf>>;

impl FileSystem for MockFileSystem {
    fn read_dir<P>(&self, dir: P) -> Result<Vec<PathBuf>>
    where
        P: AsRef<Path>,
    {
        self.get(dir.as_ref()).cloned().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no such file in mock filesystem",
            )
            .into()
        })
    }

    fn is_dir<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        self.contains_key(path.as_ref())
    }
}

impl<T> FileSystem for &T
where
    T: FileSystem,
{
    fn read_dir<P>(&self, dir: P) -> Result<Vec<PathBuf>>
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
