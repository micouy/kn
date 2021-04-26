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
    event::{Event, Key},
    input::{TermRead, TermReadEventsAndRaw},
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

mod search;
mod ui;

use search::Search;
use ui::UI;

// Helper constants.
mod sequence {
    pub const CTRL_J: [u8; 1] = [0xa];
    pub const ENTER: [u8; 1] = [0xd];
}

pub fn interactive(matches: &ArgMatches<'_>) -> Result<()> {
    let file = matches
        .value_of_os("TMP_FILE")
        .ok_or(dev_err!("required arg absent"))?;
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode()?;

    let mut search = Search::new(ReadDirEngine);
    let mut ui = UI::new(&mut stdout, vec![], "".to_string(), vec![])?;
    ui.display()?;

    for event_and_bytes in stdin.events_and_raw() {
        let (event, bytes) = event_and_bytes?;

        if let Event::Key(key) = event {
            match key {
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
                        search.get_path().ok_or(Error::NoPathFound)?;
                    fs::write(file, &*found_path.to_string_lossy())?;

                    ui.clear()?;

                    return Ok(());
                }

                // `Ctrl + c`.
                Key::Ctrl('c') => {
                    ui.clear()?;

                    return Err(Error::CtrlC);
                }

                // Any other char.
                Key::Char(c) => {
                    let (location, query, suggestions) = search.consume_char(c);
                    let suggestions = suggestions
                        .into_iter()
                        .filter_map(|Entry { path, .. }| {
                            path.file_name().map(|file_name| {
                                file_name.to_string_lossy().to_string()
                            })
                        })
                        .collect::<Vec<_>>();
                    let stdout = ui.take();
                    ui = UI::new(stdout, location, query, suggestions)?;
                    ui.display()?;
                }

                // `Backspace`.
                Key::Backspace => {
                    let (location, query, suggestions) = search.delete();
                    let suggestions = suggestions
                        .into_iter()
                        .filter_map(|Entry { path, .. }| {
                            path.file_name().map(|file_name| {
                                file_name.to_string_lossy().to_string()
                            })
                        })
                        .collect::<Vec<_>>();
                    let stdout = ui.take();
                    ui = UI::new(stdout, location, query, suggestions)?;
                    ui.display()?;
                }

                _ => {}
            }

            ui.display()?;
        }
    }

    Err(Error::NoPathFound)
}
