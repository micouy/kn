#![allow(warnings)]
#![feature(destructuring_assignment)]

use std::{
    env,
    ffi::OsString,
    fs,
    io::{stdin, stdout, Stdout, Write},
    iter,
    mem,
    path::{Path, PathBuf},
    process::exit,
};

use alphanumeric_sort;
use clap::ArgMatches;
use termion::{
    clear,
    cursor::{self, DetectCursorPos, Goto},
    event::{Event, Key},
    input::{TermRead, TermReadEventsAndRaw},
    raw::IntoRawMode,
};

use crate::{
    error::{Error, Result},
    search::{
        self,
        abbr::{Abbr, Congruence},
        fs::{DefaultFileSystem, FileSystem},
        search_level,
        Finding,
    },
    utils::{self, as_path},
};

mod ui;

use ui::{UIState, UI};

// Helper constants.
mod sequence {
    pub const CTRL_J: [u8; 1] = [0xa];
    pub const ENTER: [u8; 1] = [0xd];
}

#[derive(Debug, Clone)]
struct Location {
    root: PathBuf,
    suffix: Vec<OsString>,
    children: Vec<PathBuf>,
}

impl Location {
    fn new<F>(root: PathBuf, file_system: &F) -> Result<Self>
    where
        F: FileSystem,
    {
        let children = Self::prepare_children(&root, file_system)?;

        let location = Self {
            root,
            suffix: vec![],
            children,
        };

        Ok(location)
    }

    fn get_children(&self) -> &[PathBuf] {
        &self.children
    }

    fn prepare_children<P, F>(path: P, file_system: &F) -> Result<Vec<PathBuf>>
    where
        P: AsRef<Path>,
        F: FileSystem,
    {
        let mut children = file_system.read_dir(path)?;
        alphanumeric_sort::sort_path_slice(&mut children);

        Ok(children)
    }

    fn get_path(&self) -> PathBuf {
        let mut root = self.root.clone();

        for component in &self.suffix {
            root.push(component);
        }

        root
    }

    fn get_suffix(&self) -> PathBuf {
        self.suffix.iter().map(as_path).collect()
    }

    fn pop<F>(&mut self, file_system: &F) -> Result<bool>
    where
        F: FileSystem,
    {
        let did_pop = self.suffix.pop().is_some();
        let new_location = iter::once(as_path(&self.root))
            .chain(self.suffix.iter().map(as_path))
            .collect::<PathBuf>();
        self.children = Self::prepare_children(new_location, file_system)?;

        Ok(did_pop)
    }

    fn push<P, F>(&mut self, new_component: P, file_system: &F) -> Result<()>
    where
        P: AsRef<Path>,
        F: FileSystem,
    {
        let new_component = new_component.as_ref();
        let mut components = new_component.components();
        let first_yield = components.next();
        let second_yield = components.next();

        match (first_yield, second_yield) {
            (_, Some(_)) =>
                Err(dev_err!("attempt to push multiple components at once")),
            (None, None) => Err(dev_err!("attempt to push empty component")),
            (Some(component), None) => {
                let new_location = iter::once(as_path(&self.root))
                    .chain(self.suffix.iter().map(as_path))
                    .chain(iter::once(as_path(&component)))
                    .collect::<PathBuf>();
                self.children =
                    Self::prepare_children(new_location, file_system)?;
                self.suffix.push(component.as_os_str().to_os_string());

                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Filter {
    /// User's input.
    input: String,

    /// Children indices corresponding to suggestion indices.
    ordering: Vec<usize>,
}

impl Filter {
    fn new(input: String, children: &[PathBuf]) -> Self {
        let abbr = Abbr::from_string(input.clone()).unwrap(); // TODO
        let mut results = children
            .iter()
            .enumerate()
            .filter_map(|(ix, path)| {
                let s = path.file_name()?.to_string_lossy();
                abbr.compare(&s).map(|congruence| (ix, congruence))
            })
            .collect::<Vec<_>>();

        results.sort_by(|(_, congruence_a), (_, congruence_b)| {
            congruence_a.cmp(congruence_b)
        });
        let ordering = results.into_iter().map(|(ix, _)| ix).collect();

        Filter { input, ordering }
    }

    fn order_children<'a>(&self, children: &'a [PathBuf]) -> Vec<&'a PathBuf> {
        self.ordering
            .iter()
            .filter_map(|child_ix| children.get(*child_ix))
            .collect()
    }

    fn translate_index<'a>(&self, suggestion_ix: usize) -> Result<usize> {
        self.ordering.get(suggestion_ix).copied().ok_or(dev_err!((
            "suggestion out of bounds",
            self.clone(),
            suggestion_ix
        )))
    }
}

#[derive(Debug, Clone)]
enum Suggestions {
    All {
        selection: Option<usize>,
    },
    Filtered {
        filter: Filter,
        selection: Option<usize>,
    },
}

#[derive(Debug, Clone)]
struct State {
    location: Location,
    suggestions: Suggestions,
}

impl State {
    pub fn new<F>(root: PathBuf, file_system: &F) -> Result<Self>
    where
        F: FileSystem,
    {
        let location = Location::new(root, file_system)?;
        let state = Self {
            location,
            suggestions: Suggestions::All { selection: None },
        };

        Ok(state)
    }

    fn get_path(&self) -> Option<PathBuf> {
        // TODO: Handle errors.
        let child_ix = &match self.suggestions {
            Suggestions::All { selection } => selection,
            Suggestions::Filtered { selection, .. } => selection,
        };

        if let Some(child_ix) = child_ix {
            let child = self.location.get_children().get(*child_ix).cloned();

            child
        } else {
            let path = self.location.get_path();

            Some(path)
        }
    }

    fn handle_input<F>(&mut self, c: char, file_system: &F) -> Result<UIState>
    where
        F: FileSystem,
    {
        if c == '/' || c == '\\' {
            self.confirm_selection(file_system)?;
        } else {
            self.consume_char(c);
        }

        let ui_state = self.get_ui_state();

        Ok(ui_state)
    }

    fn select_suggestion(&mut self, suggestion_ix: usize) -> Result<()> {
        match &mut self.suggestions {
            Suggestions::All { ref mut selection } => {
                let child_ix = suggestion_ix;

                if (0..self.location.get_children().len()).contains(&child_ix) {
                    *selection = Some(child_ix);

                    Ok(())
                } else {
                    Err(dev_err!("suggestion out of bounds"))
                }
            }
            Suggestions::Filtered {
                ref mut selection,
                filter,
            } => {
                let child_ix = filter.translate_index(suggestion_ix)?;
                *selection = Some(child_ix);

                Ok(())
            }
        }
    }

    fn confirm_selection<F>(&mut self, file_system: &F) -> Result<()>
    where
        F: FileSystem,
    {
        let child_ix = match &self.suggestions {
            Suggestions::All { selection } => selection,
            Suggestions::Filtered { selection, .. } => selection,
        };
        if let Some(child_ix) = child_ix {
            let child = self
                .location
                .get_children()
                .get(*child_ix)
                .ok_or(dev_err!("child out of bounds"))?;
            let file_name = child
                .file_name()
                .ok_or(dev_err!("no file name"))?
                .to_owned();
            self.location.push(file_name, file_system)?;
        }

        self.suggestions = Suggestions::All { selection: None };

        Ok(())
    }

    fn consume_char(&mut self, c: char) {
        let new_input = match &self.suggestions {
            Suggestions::All { .. } => c.to_string(),
            Suggestions::Filtered {
                filter: Filter { input, .. },
                selection,
            } => {
                let mut new_input = input.clone();
                new_input.push(c);

                new_input
            }
        };

        let filter = Filter::new(new_input, &self.location.children);
        self.suggestions = Suggestions::Filtered {
            filter,
            selection: None,
        };
    }

    fn handle_backspace<F>(&mut self, file_system: &F) -> Result<UIState>
    where
        F: FileSystem,
    {
        match &mut self.suggestions {
            Suggestions::Filtered { .. } =>
                self.suggestions = Suggestions::All { selection: None },
            Suggestions::All { ref mut selection } => {
                let _ = self.location.pop(file_system)?;
                *selection = None;
            }
        }

        let ui_state = self.get_ui_state();

        Ok(ui_state)
    }

    fn get_ui_state(&self) -> UIState {
        let input = match &self.suggestions {
            Suggestions::Filtered { filter, .. } => Some(filter.input.clone()),
            _ => None,
        };
        let location = self.location.get_suffix();

        let suggestions = match &self.suggestions {
            Suggestions::All { .. } => self
                .location
                .get_children()
                .iter()
                .filter_map(|child| child.file_name())
                .map(|file_name| file_name.to_string_lossy().into_owned())
                .collect::<Vec<String>>(),
            Suggestions::Filtered { filter, .. } => filter
                .order_children(self.location.get_children())
                .iter()
                .filter_map(|child| child.file_name())
                .map(|file_name| file_name.to_string_lossy().into_owned())
                .collect::<Vec<String>>(),
        };

        UIState {
            input,
            location,
            suggestions,
        }
    }
}

pub fn interactive() -> Result<PathBuf> {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode()?;
    // let mut stdout = termion::screen::AlternateScreen::from(stdout);
    let root = env::current_dir()?;
    let file_system = DefaultFileSystem;

    let mut state = State::new(root, &file_system)?;
    let ui_state = state.get_ui_state();
    let (mut ui, selection) = UI::new(&mut stdout, ui_state)?;

    if let Some(suggestion_ix) = selection {
        state.select_suggestion(suggestion_ix);
    }

    ui.clear()?;
    ui.display()?;

    for event_and_bytes in stdin.events_and_raw() {
        let (event, bytes) = event_and_bytes?;

        if let Event::Key(key) = event {
            let suggestion_ix = match key {
                // `Ctrl + h` and `Ctrl + l`.
                Key::Ctrl('h') => ui.previous_suggestion(),
                Key::Ctrl('l') => ui.next_suggestion(),

                // `Tab` and `Shift + Tab`.
                Key::BackTab => ui.previous_suggestion(),
                Key::Char('\t') => ui.next_suggestion(),

                // `Ctrl + j` and `Ctrl + k`.
                Key::Ctrl('k') => ui.previous_page(),
                _ if bytes == sequence::CTRL_J => ui.next_page(),

                // `Enter`.
                _ if bytes == sequence::ENTER => {
                    let found_path =
                        state.get_path().ok_or(Error::NoPathFound)?;
                    ui.clear()?;
                    drop(ui.take());

                    return Ok(found_path.clone());
                }

                // `Ctrl + c`.
                Key::Ctrl('c') => {
                    ui.clear()?;

                    return Err(Error::CtrlC);
                }

                // Any other char, excluding whitespace.
                Key::Char(c) if !c.is_whitespace() => {
                    let ui_state = state.handle_input(c, &file_system)?;
                    let stdout = ui.take();
                    let ui_and_selection = UI::new(stdout, ui_state)?;
                    ui = ui_and_selection.0;
                    let selection = ui_and_selection.1;

                    selection
                }

                // `Backspace`.
                Key::Backspace => {
                    let ui_state = state.handle_backspace(&file_system)?;
                    let stdout = ui.take();
                    let ui_and_selection = UI::new(stdout, ui_state)?;
                    ui = ui_and_selection.0;
                    let selection = ui_and_selection.1;

                    selection
                }

                _ => None,
            };

            if let Some(suggestion_ix) = suggestion_ix {
                state.select_suggestion(suggestion_ix);
            }

            ui.display()?;
        }
    }

    Err(Error::NoPathFound)
}

fn main() {
    match interactive() {
        Err(err) => println!("{}", err),
        _ => {}
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

            let filter = Filter::new("a".to_string(), &children);
            let ordered = filter.order_children(&children);

            assert_eq!(ordered, vec![a, abc]);
        }

        {
            let children: Vec<PathBuf> = vec!["abc", "a", "smth_else"]
                .iter()
                .map(PathBuf::from)
                .collect();

            let a = &children[1];
            let abc = &children[0];

            let filter = Filter::new("a".to_string(), &children);
            let ordered = filter.order_children(&children);

            assert_eq!(ordered, vec![a, abc]);
        }

        {
            let children: Vec<PathBuf> = vec!["smth_else", "abc", "a"]
                .iter()
                .map(PathBuf::from)
                .collect();

            let a = &children[2];
            let abc = &children[1];

            let filter = Filter::new("a".to_string(), &children);
            let ordered = filter.order_children(&children);

            assert_eq!(ordered, vec![a, abc]);
        }
    }
}
