#![allow(dead_code)]

use std::{
    fs,
    io::{stdin, stdout, Write},
    mem,
    path::PathBuf,
    process::exit,
};
use termion::{
    clear,
    cursor::{self, DetectCursorPos, Goto},
    event::Key,
    input::TermRead,
    raw::IntoRawMode,
};

#[macro_use]
mod utils;
mod error;
mod query;

use query::{
    abbr::{Abbr, Congruence},
    entry::{Entry, Flow},
    search_engine::{ReadDirEngine, SearchEngine},
};

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

fn main() {
    let file = std::env::args().nth(1).unwrap();
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    // Make room for input and results.
    write!(stdout, "\n").unwrap();
    stdout.flush().unwrap();

    write!(stdout, "{}", cursor::Up(1)).unwrap();
    stdout.flush().unwrap();

    let mut search = Search::new(ReadDirEngine);

    for c in stdin.keys() {
        let results = match c.unwrap() {
            Key::Ctrl('c') => {
                let current_line = stdout.cursor_pos().unwrap().1;
                write!(
                    stdout,
                    "{}{}",
                    cursor::Goto(1, current_line),
                    clear::AfterCursor,
                )
                .unwrap();
                stdout.flush().unwrap();

                exit(1);
            }
            Key::Char('\n') => {
                let found_path = search.get_path().unwrap();
                fs::write(file, &*found_path.to_string_lossy()).unwrap();

                let current_line = stdout.cursor_pos().unwrap().1;
                write!(
                    stdout,
                    "{}{}",
                    cursor::Goto(1, current_line),
                    clear::AfterCursor,
                )
                .unwrap();
                stdout.flush().unwrap();

                exit(0);
            }
            Key::Char(c) => Some(search.consume_char(c)),
            Key::Backspace => Some(search.delete()),
            _ => None,
        };

        if let Some((string, findings)) = results {
            let current_line = stdout.cursor_pos().unwrap().1;
            write!(
                stdout,
                "{}{}{}",
                clear::CurrentLine,
                cursor::Goto(1, current_line),
                string,
            )
            .unwrap();

            if let Some(finding) = findings.get(0) {
                write!(stdout, "{}{}", cursor::Save, cursor::Down(1)).unwrap();
                let current_line = stdout.cursor_pos().unwrap().1;
                write!(
                    stdout,
                    "{}{}{}{}",
                    cursor::Goto(1, current_line),
                    clear::CurrentLine,
                    finding.path.display(),
                    cursor::Restore,
                )
                .unwrap();
            } else {
                write!(stdout, "{}{}", cursor::Save, cursor::Down(1)).unwrap();
                let current_line = stdout.cursor_pos().unwrap().1;
                write!(
                    stdout,
                    "{}{}{}",
                    cursor::Goto(1, current_line),
                    clear::CurrentLine,
                    cursor::Restore,
                )
                .unwrap();
            }

            stdout.flush().unwrap();
        }
    }
}

fn print_abbr(abbr: &[Finding], last: &str) -> String {
    let abbr = abbr
        .iter()
        .map(|Finding { abbr, .. }| match abbr {
            Abbr::Literal(s) => s,
            Abbr::Wildcard => "-",
        })
        .fold(String::new(), |abbr, component| abbr + component + "/");

    abbr + last
}

enum Prefix {
    RootDir,
    CurrentDir,
    ParentDir,
    HomeDir,
}

struct Finding {
    abbr: Abbr,
    entries: Vec<Entry>,
}

struct Search<E>
where
    E: SearchEngine,
{
    engine: E,
    input: String,
    findings: Vec<Finding>,
    current_level: Vec<Entry>,
}

impl<E> Search<E>
where
    E: SearchEngine,
{
    fn new(engine: E) -> Self {
        let current_level = engine
            .read_dir(".")
            .into_iter()
            .map(|path| Entry::new(path, vec![]))
            .collect();

        Self {
            input: String::new(),
            findings: vec![],
            current_level,
            engine,
        }
    }

    // NA RAZIE ZUPEŁNIE IGNOROWAĆ PREFIX ../.././ ITD
    // ABBR powinno brać reference tylko
    fn consume_char(&mut self, c: char) -> (String, Vec<Entry>) {
        let findings = if c == '/' {
            if !self.input.is_empty() {
                // Perhaps repeating the search is unnecessary. It would
                // be enough to cache the previous search and just push it to
                // findings.
                let input = mem::replace(&mut self.input, String::new());
                let abbr = Abbr::from_string(input).unwrap();

                // Get matching entries and order them.
                let mut entries: Vec<_> = self
                    .current_level
                    .iter()
                    .filter_map(|entry| match entry.advance(&abbr) {
                        Flow::DeadEnd => None,
                        Flow::Continue(entry) => Some(entry),
                    })
                    .collect();
                entries.sort_by(|a, b| a.congruence.cmp(&b.congruence));

                // Fill current level with children of the previous one.
                self.current_level.clear();
                let engine = &self.engine;
                self.current_level.extend(
                    entries
                        .iter()
                        .map(|Entry { path, congruence }| {
                            engine.read_dir(path).into_iter().map(move |path| {
                                Entry {
                                    path,
                                    congruence: congruence.clone(),
                                }
                            })
                        })
                        .flatten(),
                );

                self.findings.push(Finding { abbr, entries });
            }

            self.findings
                .last()
                .map(|Finding { entries, .. }| entries.clone())
                .unwrap_or_else(|| vec![])
        } else {
            // Construct a new abbr.
            self.input.push(c);
            let abbr = Abbr::from_string(self.input.clone()).unwrap();

            // Get matching entries and order them.
            let mut entries: Vec<_> = self
                .current_level
                .iter()
                .filter_map(|entry| match entry.advance(&abbr) {
                    Flow::DeadEnd => None,
                    Flow::Continue(entry) => Some(entry),
                })
                .collect();
            entries.sort_by(|a, b| a.congruence.cmp(&b.congruence));

            entries
        };

        let input = print_abbr(&self.findings, &self.input);

        (input, findings)
    }

    fn get_path(&self) -> Option<PathBuf> {
        let abbr = Abbr::from_string(self.input.clone()).unwrap();

        // Get matching entries and order them.
        let mut entries: Vec<_> = self
            .current_level
            .iter()
            .filter_map(|entry| match entry.advance(&abbr) {
                Flow::DeadEnd => None,
                Flow::Continue(entry) => Some(entry),
            })
            .collect();
        entries.sort_by(|a, b| a.congruence.cmp(&b.congruence));

		if entries.get(0).is_some() {
        	Some(entries.remove(0).path)
		} else {
    		None
		}
    }

    fn delete(&mut self) -> (String, Vec<Entry>) {
        if self.input.is_empty() {
            let _ = self.findings.pop();
            let _ = self.input.pop();

            // Fill current level with children of the previous one.
            let root_entry = vec![Entry::new(".".into(), vec![])];
            let entries = self
                .findings
                .last()
                .map(|Finding { entries, .. }| entries)
                .unwrap_or(&root_entry);
            self.current_level.clear();
            let engine = &self.engine;
            self.current_level.extend(
                entries
                    .iter()
                    .map(|Entry { path, congruence }| {
                        engine.read_dir(path).into_iter().map(move |path| {
                            Entry {
                                path,
                                congruence: congruence.clone(),
                            }
                        })
                    })
                    .flatten(),
            );
        } else {
            self.input.clear();
        }

        let input = print_abbr(&self.findings, &self.input);

        (input, vec![])
    }
}
