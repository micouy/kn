use std::{
    convert::AsRef,
    iter,
    mem,
    path::{Path, PathBuf},
};

pub mod abbr;
pub mod fs;

use abbr::*;
use fs::FileSystem;

#[derive(Debug, Clone)]
pub struct Finding {
    pub path: PathBuf,
    pub congruence: Congruence,
}

pub fn search_full<'a, P, I, F>(
    root: P,
    mut abbrs: I,
    file_system: &F,
) -> Vec<PathBuf>
where
    P: AsRef<Path>,
    I: Iterator<Item = &'a Abbr>,
    F: FileSystem,
{
    let mut current_level: Vec<(PathBuf, Vec<Congruence>)> =
        if let Some(first_abbr) = abbrs.next() {
            let children = get_children(iter::once(root), file_system);
            let findings = filter_paths(children, first_abbr)
                .map(|finding| (finding.path, vec![finding.congruence]))
                .collect();

            findings
        } else {
            return vec![];
        };

    let mut next_level: Vec<(PathBuf, Vec<Congruence>)> = vec![];

    for abbr in abbrs {
        next_level.clear();

        for (path, congruence) in current_level.drain(..) {
            let children =
                get_children(iter::once(path.as_path()), file_system);
            let paths = filter_paths(children, abbr).map(|finding| {
                let mut new_congruence = congruence.clone();
                new_congruence.push(finding.congruence);

                (finding.path, new_congruence)
            });

            next_level.extend(paths);
        }

        mem::swap(&mut current_level, &mut next_level);
    }

    current_level.sort_by(|a, b| a.1.cmp(&b.1));
    let findings = current_level
        .into_iter()
        .map(|(path, _congruence)| path)
        .collect();

    findings
}

pub fn get_children<'a, I, P, F>(
    paths: I,
    file_system: &'a F,
) -> impl Iterator<Item = PathBuf> + 'a
where
    I: Iterator<Item = P> + 'a,
    P: AsRef<Path> + 'a,
    F: FileSystem + 'a,
{
    paths
        .filter_map(move |path| file_system.read_dir(path).ok())
        .flatten()
}

pub fn filter_paths<'a, I>(
    paths: I,
    abbr: &'a Abbr,
) -> impl Iterator<Item = Finding> + 'a
where
    I: Iterator<Item = PathBuf> + 'a,
{
    paths.filter_map(move |path| {
        let file_name = path.file_name()?.to_string_lossy();
        let congruence = abbr.compare(&file_name)?;
        let finding = Finding { path, congruence };

        Some(finding)
    })
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::utils::as_path;

    #[test]
    fn test_search_full() {
        use std::collections::HashMap;

        let mut file_system: fs::MockFileSystem = HashMap::new();
        file_system.insert(".".into(), vec!["./ex".into()]);
        file_system
            .insert("./ex".into(), vec!["./ex/dee".into(), "./ex/why".into()]);
        file_system.insert(
            "./ex/dee".into(),
            vec!["./ex/dee/dee".into(), "./ex/dee/deedee".into()],
        );

        let abbrs = vec![
            Abbr::from_string("x".to_string()).unwrap(),
            Abbr::from_string("d".to_string()).unwrap(),
            Abbr::from_string("d".to_string()).unwrap(),
        ];

        let found_path = search_full(".", abbrs.iter(), &file_system).remove(0);

        assert_eq!(as_path("./ex/dee/dee"), found_path);
    }
}
