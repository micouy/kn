use clap::ArgMatches;
use std::{
    fs,
    io::{stdin, stdout, Stdout, Write},
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

use crate::{
    error::{Error, Result},
    query::{
        abbr::{Abbr, Congruence},
        entry::{Entry, Flow},
        search_engine::{ReadDirEngine, SearchEngine},
    },
};

mod ui;

pub fn interactive(matches: &ArgMatches<'_>) -> Result<()> {
    let file = matches
        .value_of_os("TMP_FILE")
        .ok_or(dev_err!("required arg absent"))?;
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode()?;

    // Make room for input and results.
    write!(stdout, "\n")?;
    stdout.flush()?;

    write!(stdout, "{}", cursor::Up(1))?;
    stdout.flush()?;

    let mut search = Search::new(ReadDirEngine);

    for c in stdin.keys() {
        let results = match c? {
            Key::Ctrl('c') => {
                let current_line = stdout.cursor_pos()?.1;
                write!(
                    stdout,
                    "{}{}",
                    cursor::Goto(1, current_line),
                    clear::AfterCursor,
                )?;
                stdout.flush()?;

                return Err(Error::CtrlC);
            }
            Key::Char('\n') => {
                let found_path = search.get_path().ok_or(Error::NoPathFound)?;
                fs::write(file, &*found_path.to_string_lossy())?;

                let current_line = stdout.cursor_pos()?.1;
                write!(
                    stdout,
                    "{}{}",
                    cursor::Goto(1, current_line),
                    clear::AfterCursor,
                )?;
                stdout.flush()?;

                return Ok(());
            }
            Key::Char(c) => Some(search.consume_char(c)),
            Key::Backspace => Some(search.delete()),
            _ => None,
        };

        if let Some((string, findings)) = results {
            print_state(string, findings, &mut stdout)?;
        }
    }

    Err(Error::NoPathFound)
}

fn print_state(
    query: String,
    findings: Vec<Entry>,
    stdout: &mut Stdout,
) -> Result<()> {
    let current_line = stdout.cursor_pos()?.1;
    write!(
        stdout,
        "{}{}{}",
        clear::CurrentLine,
        cursor::Goto(1, current_line),
        query,
    )?;

    if let Some(finding) = findings.get(0) {
        write!(stdout, "{}{}", cursor::Save, cursor::Down(1))?;
        let current_line = stdout.cursor_pos()?.1;
        write!(
            stdout,
            "{}{}{}{}",
            cursor::Goto(1, current_line),
            clear::CurrentLine,
            finding.path.display(),
            cursor::Restore,
        )?;
    } else {
        write!(stdout, "{}{}", cursor::Save, cursor::Down(1))?;
        let current_line = stdout.cursor_pos()?.1;
        write!(
            stdout,
            "{}{}{}",
            cursor::Goto(1, current_line),
            clear::CurrentLine,
            cursor::Restore,
        )?;
    }

    stdout.flush()?;

    Ok(())
}

// TODO: Rename.
// TODO: Make this code much more declarative.
fn print_abbr(abbr: &[Finding], last: &str) -> String {
    let start_ix = abbr.len().saturating_sub(4);
    let end_ix = abbr.len();
    let prefix = if start_ix == 0 {
        ""
    } else {
        "…/"
    }.to_string();
    let abbr = abbr[start_ix..end_ix]
        .iter()
        .map(|Finding { abbr, .. }| match abbr {
            Abbr::Literal(s) => s,
            Abbr::Wildcard => "-",
        })
        .fold(String::new(), |abbr, component| abbr + component + "/");

    prefix + &abbr + last
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

pub enum SearchResults<'a> {
    Findings(&'a [Entry]),
    Suggestions(&'a [Entry]),
}
