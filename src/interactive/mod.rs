use std::{
    env,
    io::{stdin, stdout, Write},
    path::{Path, PathBuf},
};

use termion::{
    event::{Event, Key},
    input::TermReadEventsAndRaw,
    raw::IntoRawMode,
};

use crate::{
    error::{Error, Result},
    search::{
        abbr::Abbr,
        fs::{DefaultFileSystem, FileSystem},
    },
};

mod ui;

use ui::UI;

// Helper constants.
mod sequence {
    pub const CTRL_J: [u8; 1] = [0xa];
    pub const ENTER: [u8; 1] = [0xd];
}

#[derive(Debug, Clone)]
struct Location {
    root: PathBuf,
    history: Vec<PathBuf>,
}

impl Location {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            history: vec![],
        }
    }

    fn get_path(&self) -> PathBuf {
        self.history.last().unwrap_or(&self.root).clone()
    }

    fn get_suffix(&self) -> PathBuf {
        let location = self.history.last().unwrap_or(&self.root);

        let location = match location.strip_prefix(&self.root) {
            Ok(suffix) => suffix,
            Err(_) => location,
        };

        let components =
            location.components().rev().take(2).collect::<Vec<_>>();
        components.into_iter().rev().collect()
    }

    fn pop(&mut self) -> bool {
        self.history.pop().is_some()
    }

    fn push<P>(&mut self, new_location: P)
    where
        P: AsRef<Path>,
    {
        self.history.push(new_location.as_ref().into());
    }
}

#[derive(Debug, Clone)]
struct Filter {
    input: Option<(String, Abbr)>,
}

impl Filter {
    fn new() -> Self {
        Filter { input: None }
    }

    fn push(&mut self, c: char) -> Result<()> {
        match &mut self.input {
            Some((ref mut input, ref mut abbr)) => {
                input.push(c);
                *abbr = Abbr::from_string(input.clone())?;

                Ok(())
            }
            None => {
                let input = c.to_string();
                let abbr = Abbr::from_string(input.clone())?;
                self.input = Some((input, abbr));

                Ok(())
            }
        }
    }

    fn take(&mut self) -> Option<String> {
        self.input.take().map(|(input, _)| input)
    }

    fn get_input(&self) -> Option<&String> {
        self.input.as_ref().map(|(input, _)| input)
    }

    fn filter<'a>(&self, children: &'a [PathBuf]) -> Vec<&'a PathBuf> {
        match &self.input {
            Some((_, abbr)) => {
                let mut results = children
                    .iter()
                    .filter_map(|path| {
                        let s = path.file_name()?.to_string_lossy();
                        abbr.compare(&s).map(|congruence| (path, congruence))
                    })
                    .collect::<Vec<_>>();

                results.sort_by(
                    |(path_a, congruence_a), (path_b, congruence_b)| {
                        let by_congruence = congruence_a.cmp(congruence_b);
                        let by_alphanumeric =
                            alphanumeric_sort::compare_path(path_a, path_b);

                        by_congruence.then(by_alphanumeric)
                    },
                );

                results.into_iter().map(|(path, _)| path).collect()
            }
            None => {
                let mut children = children.iter().collect::<Vec<_>>();
                alphanumeric_sort::sort_path_slice(&mut children);

                children
            }
        }
    }
}

pub fn interactive() -> Result<PathBuf> {
    let stdout = stdout();
    let mut stdout = stdout.into_raw_mode()?;

    let result = _interactive(&mut stdout);
    stdout.suspend_raw_mode()?;

    write!(&mut stdout, "\r{}", termion::clear::AfterCursor)?;
    stdout.flush()?;

    result
}

fn prepare_ui<'a, W, F>(
    location: &Location,
    filter: &Filter,
    file_system: &F,
    stdout: &'a mut W,
) -> Result<UI<'a, W>>
where
    W: Write,
    F: FileSystem,
{
    let children = file_system.read_dir(location.get_path())?;
    let suggestions = filter.filter(&children).into_iter().cloned().collect();
    let ui = UI::new(
        location.get_suffix(),
        filter.get_input().map(|s| s.to_string()),
        suggestions,
        stdout,
    )?;

    Ok(ui)
}

fn handle_backspace(location: &mut Location, filter: &mut Filter) {
    let taken = filter.take();

    if taken.is_none() {
        let _ = location.pop();
    }
}

fn handle_slash<W>(location: &mut Location, filter: &mut Filter, ui: &UI<'_, W>) where W: Write {
    if let Some(path) = ui.get_selected_suggestion() {
        filter.take();
        location.push(path);
    }
}

fn handle_char(filter: &mut Filter, c: char) -> Result<()> {
    filter.push(c)
}

pub fn _interactive<W>(stdout: &mut W) -> Result<PathBuf>
where
    W: Write,
{
    let stdin = stdin();
    let root = env::current_dir()?;
    let file_system = DefaultFileSystem;

    let mut location = Location::new(root);
    let mut filter = Filter::new();

    let mut ui = prepare_ui(&location, &filter, &file_system, stdout)?;

    ui.display()?;

    for event_and_bytes in stdin.events_and_raw() {
        let (event, bytes) = event_and_bytes?;

        if let Event::Key(key) = event {
            match key {
                // Ctrl + h and Ctrl + l.
                Key::Ctrl('h') => ui.previous_suggestion(),
                Key::Ctrl('l') => ui.next_suggestion(),

                // Tab and Shift + Tab.
                Key::BackTab => ui.previous_suggestion(),
                Key::Char('\t') => ui.next_suggestion(),

                // Ctrl + j and Ctrl + k.
                Key::Ctrl('k') => ui.previous_page(),
                _ if bytes == sequence::CTRL_J => ui.next_page(),

                // Enter.
                _ if bytes == sequence::ENTER => {
                    let found_path = ui
                        .get_selected_suggestion()
                        .ok_or(Error::NoPathFound)?;
                    ui.clear()?;

                    return Ok(found_path);
                }

                // Ctrl + c.
                Key::Ctrl('c') => {
                    ui.clear()?;

                    return Err(Error::CtrlC);
                }

                Key::Char('/') | Key::Char('\\') => {
                    handle_slash(&mut location, &mut filter, &ui);
                }

                // Any other char, excluding whitespace.
                Key::Char(c) if !c.is_whitespace() => {
                    handle_char(&mut filter, c)?;
                }

                // Backspace.
                Key::Backspace => {
                    handle_backspace(&mut location, &mut filter);
                }
                _ => {}
            }

            let stdout = ui.take();
            ui = prepare_ui(&location, &filter, &file_system, stdout)?;
            ui.display()?;
        }
    }

    Err(Error::NoPathFound)
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::utils::as_path;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_filter() {
        {
            let children: Vec<PathBuf> = vec!["a", "abc", "smth_else"]
                .iter()
                .map(PathBuf::from)
                .collect();

            let a = &children[0];
            let abc = &children[1];

            let mut filter = Filter::new();
            filter.push('a');
            let filtered = filter.filter(&children);

            assert_eq!(filtered, vec![a, abc]);
        }

        {
            let children: Vec<PathBuf> = vec!["abc", "a", "smth_else"]
                .iter()
                .map(PathBuf::from)
                .collect();

            let a = &children[1];
            let abc = &children[0];

            let mut filter = Filter::new();
            filter.push('a');
            let filtered = filter.filter(&children);

            assert_eq!(filtered, vec![a, abc]);
        }

        {
            let children: Vec<PathBuf> = vec!["smth_else", "abc", "a"]
                .iter()
                .map(PathBuf::from)
                .collect();

            let a = &children[2];
            let abc = &children[1];

            let mut filter = Filter::new();
            filter.push('a');
            let filtered = filter.filter(&children);

            assert_eq!(filtered, vec![a, abc]);
        }
    }

    #[test]
    fn test_location_push_pop() {
        let mut location = Location::new(".".into());
        location.push("a");
        location.push("b");

        assert!(location.pop());
        assert!(location.pop());
        assert!(!location.pop());
        assert!(location.get_path() == as_path("."));
    }
}
