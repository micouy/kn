use std::{
    io::{stdin, stdout, Stdout, Write},
    mem,
    ops::RangeBounds,
};

use termion::{
    clear,
    color,
    cursor::{self, DetectCursorPos},
    event::{Event, Key},
    input::{TermRead, TermReadEventsAndRaw},
    raw::{IntoRawMode, RawTerminal},
};

use unicode_width::*;

use crate::error::{Error, Result};

const SUGGESTIONS_SEPARATOR: &'static str = "  ";

// Palette:
// https://coolors.co/9c71f3-47f0a7-cca6e8-8380b6-111d4a

/*
ASSUMPTION 1: Width (the number of cells occupied) of these strings
will not change (or at least not grow) depending on the
renderer used and what characters surround them. This means
the width of the displayed string is the sum of the widths of
the substrings (or at least that it's not greater).
*/

/*
ASSUMPTION 2: When`display()` is called, the program assumes that the cursor is
in the proper position to start printing.
*/

pub struct UI<'a> {
    stdout: &'a mut RawTerminal<Stdout>,
    input: Input,
    pages: Option<Pages>,
}

impl<'a> UI<'a> {
    pub fn new(
        stdout: &'a mut RawTerminal<Stdout>,
        location: Vec<String>,
        query: String,
        suggestions: Vec<String>,
    ) -> Result<Self> {
        let (width, _) = termion::terminal_size()?;

        let pages = if suggestions.is_empty() {
            None
        } else {
            let pages = Pages::new(suggestions, width as usize)?;

            Some(pages)
        };
        let input = Input::new(location, query);

        let ui = Self {
            stdout,
            input,
            pages,
        };

        Ok(ui)
    }

    pub fn update(&mut self,
        location: Vec<String>,
        query: String,
        suggestions: Vec<String>,
    ) -> Result<()> {
        let (width, _) = termion::terminal_size()?;

        let pages = if suggestions.is_empty() {
            None
        } else {
            let pages = Pages::new(suggestions, width as usize)?;

            Some(pages)
        };
        let input = Input::new(location, query);

        self.pages = pages;
        self.input = input;

        Ok(())
    }

    pub fn display(&mut self) -> Result<()> {
        // Assuming cursor is at the original input line, not necessarily at the
        // first char.

        // Make room for input and results.
        write!(
            self.stdout,
            "{clear}\n{clear}{up}",
            clear = clear::AfterCursor,
            up = cursor::Up(1),
        )?;
        self.stdout.flush()?;

        let current_line = Cursor::get_line(&mut self.stdout)?;
        write!(
            self.stdout,
            "{}{}",
            cursor::Goto(1, current_line),
            clear::AfterCursor
        )?;

        self.input.display(&mut self.stdout)?;
        let cursor_pos = Cursor::save(&mut self.stdout)?;
        write!(self.stdout, "{}", cursor::Down(1))?;

        if let Some(ref mut pages) = self.pages {
            pages.display(&mut self.stdout)?;
        }

        cursor_pos.restore(&mut self.stdout)?;

        Ok(())
    }

    pub fn next_suggestion(&mut self) {
        if let Some(ref mut pages) = self.pages {
            pages.next_suggestion();
        }
    }

    pub fn previous_suggestion(&mut self) {
        if let Some(ref mut pages) = self.pages {
            pages.previous_suggestion();
        }
    }

    pub fn next_page(&mut self) {
        if let Some(ref mut pages) = self.pages {
            pages.next_page();
        }
    }

    pub fn previous_page(&mut self) {
        if let Some(ref mut pages) = self.pages {
            pages.previous_page();
        }
    }

    pub fn clear(&mut self) -> Result<()> {
        let current_line = Cursor::get_line(self.stdout)?;
        write!(
            self.stdout,
            "{}{}",
            cursor::Goto(1, current_line),
            clear::AfterCursor,
        )?;
        self.stdout.flush()?;

        Ok(())
    }

    pub fn take(self) -> &'a mut RawTerminal<Stdout> {
        self.stdout
    }
}

struct Cursor;

impl Cursor {
    fn save<W>(stdout: &mut W) -> Result<Self>
    where
        W: Write,
    {
        write!(stdout, "{}", cursor::Save)?;
        stdout.flush()?;

        Ok(Self)
    }

    fn restore<W>(self, stdout: &mut W) -> Result<()>
    where
        W: Write,
    {
        write!(stdout, "{}", cursor::Restore)?;
        stdout.flush()?;

        Ok(())
    }

    fn get_line<W>(stdout: &mut W) -> Result<u16>
    where
        W: Write,
    {
        stdout.flush()?;
        let (_, line) = stdout.cursor_pos()?;

        Ok(line)
    }
}
struct Input {
    query: String,
    location: Vec<String>,
}

impl Input {
    fn new(location: Vec<String>, query: String) -> Self {
        Self { location, query }
    }

    fn display<W>(&self, stdout: &mut W) -> Result<()>
    where
        W: Write,
    {
        let location = Self::compose_location(&self.location);
        let current_line = Cursor::get_line(stdout)?;

        write!(
            stdout,
            "{clear}{goto}{location_fg}{location}{query_fg}{query}{reset_fg}{reset_bg}",
            clear = clear::CurrentLine,
            goto = cursor::Goto(1, current_line),
            location_fg = color::Fg(color::AnsiValue::grayscale(16)),
            location = location,
            query_fg = color::Fg(color::Rgb(156, 113, 243)),
            query = self.query,
            reset_fg = color::Fg(color::Reset),
            reset_bg = color::Bg(color::Reset),
        )?;
        stdout.flush()?;

        Ok(())
    }

    fn compose_location(location: &[String]) -> String {
        let start_ix = location.len().saturating_sub(2);
        let end_ix = location.len();
        let prefix = if start_ix == 0 { "" } else { ".../" }.to_string();
        let location = location[start_ix..end_ix]
            .iter()
            .fold(String::new(), |location, component| {
                location + component + "/"
            });

        prefix + &location
    }
}

impl Page {
    fn display<W>(
        &self,
        selected_ix: usize,
        suggestions: &[String],
        stdout: &mut W,
    ) -> Result<()>
    where
        W: Write,
    {
        Ok(())
    }
}

// TODO: Handle the first suggestion separately.
//   - No space at the beginning.
//   - If the suggestion's length exceeds the available space, it must be
//     formatted differently.

#[derive(Copy, Clone)]
struct Page {
    start_ix: usize,
}

struct Pages {
    pages: Vec<Page>,
    suggestions: Vec<String>,
    suggestion_ix: usize,
    page_ix: usize,
}

impl Pages {
    fn new(suggestions: Vec<String>, width: usize) -> Result<Self> {
        if suggestions.is_empty() {
            Err(dev_err!("empty pages"))
        } else {
            let pages = Self::build_pages(&suggestions, width);

            let pages = Self {
                pages,
                suggestions,
                suggestion_ix: 0,
                page_ix: 0,
            };

            Ok(pages)
        }
    }

    fn display<W>(&self, stdout: &mut W) -> Result<()>
    where
        W: Write,
    {
        let suggestions = Self::get_page_suggestions(
            self.page_ix,
            &self.pages,
            &self.suggestions,
        )?;
        let page = self
            .pages
            .get(self.page_ix)
            .ok_or(dev_err!("page out of bounds"))?;
        let relative_ix = self
            .suggestion_ix
            .checked_sub(page.start_ix)
            .ok_or(dev_err!())?;
        let output = utils::compose_page(suggestions, relative_ix);
        let current_line = Cursor::get_line(stdout)?;

        write!(
            stdout,
            "{goto}{page_ix_fg}({page_ix}){reset_fg}{reset_bg}",
            goto = cursor::Goto(1, current_line),
            page_ix_fg = color::Fg(color::Rgb(71, 240, 167)),
            page_ix = self.page_ix,
            reset_fg = color::Fg(color::Reset),
            reset_bg = color::Bg(color::Reset),
        )?;

        let suggestions_x =
            1 + utils::page_ix_message_space(self.page_ix) as u16;
        write!(
            stdout,
            "{}{}",
            cursor::Goto(suggestions_x, current_line),
            output
        )?;

        stdout.flush()?;
        Ok(())
    }

    fn get_page_suggestions<'s>(
        page_ix: usize,
        pages: &[Page],
        suggestions: &'s [String],
    ) -> Result<&'s [String]> {
        let Page { start_ix } =
            pages.get(page_ix).ok_or(dev_err!("page out of bounds"))?;
        let next_page_ix = page_ix + 1;

        match pages.get(next_page_ix) {
            Some(Page { start_ix: end_ix }) => suggestions
                .get(*start_ix..*end_ix)
                .ok_or(dev_err!("suggestion out of bounds")),
            None => suggestions
                .get(*start_ix..)
                .ok_or(dev_err!("suggestion out of bounds")),
        }
    }

    fn build_pages(suggestions: &[String], space: usize) -> Vec<Page> {
        // TODO: Handle filenames exceeding terminal width;
        if suggestions.is_empty() {
            return vec![];
        }

        let mut page_width = 0;
        let mut pages = vec![Page { start_ix: 0 }];
        let mut page_space = space - utils::page_ix_message_space(pages.len());

        for (i, suggestion) in suggestions.iter().enumerate() {
            let delta_width =
                SUGGESTIONS_SEPARATOR.width() + suggestion.width();
            let new_width = page_width + delta_width;

            if new_width > page_space {
                pages.push(Page { start_ix: i });
                page_width = delta_width;
                page_space = space - utils::page_ix_message_space(pages.len());
            } else {
                page_width = new_width;
            }
        }

        pages
    }

    fn next_suggestion(&mut self) -> usize {
        let len = self.suggestions.len();
        self.suggestion_ix = (self.suggestion_ix + 1) % len;
        self.page_ix =
            Self::find_selected_page(self.suggestion_ix, &self.pages);

        self.suggestion_ix
    }

    fn previous_suggestion(&mut self) -> usize {
        let len = self.suggestions.len();
        self.suggestion_ix = (self.suggestion_ix + len - 1) % len;
        self.page_ix =
            Self::find_selected_page(self.suggestion_ix, &self.pages);

        self.suggestion_ix
    }

    fn next_page(&mut self) -> usize {
        self.page_ix = (self.page_ix + 1) % self.pages.len();
        let page = self.pages.get(self.page_ix).unwrap();
        self.suggestion_ix = page.start_ix;

        self.suggestion_ix
    }

    fn previous_page(&mut self) -> usize {
        let len = self.pages.len();
        self.page_ix = (self.page_ix + len - 1) % len;
        let page = self.pages.get(self.page_ix).unwrap();
        self.suggestion_ix = page.start_ix;

        self.suggestion_ix
    }

    fn find_selected_page(selected_ix: usize, pages: &[Page]) -> usize {
        let is_selected = |pages: &[Page]| {
            if let [page_a, page_b] = pages {
                (page_a.start_ix..page_b.start_ix).contains(&selected_ix)
            } else {
                false
            }
        };

        let last_page_ix = pages.len() - 1;

        let selected_page = pages
            .windows(2)
            .position(is_selected)
            .unwrap_or(last_page_ix);

        selected_page
    }
}

mod utils {
    use super::*;

    pub fn compose_page(suggestions: &[String], selected_ix: usize) -> String {
        let is_selected = |ix| selected_ix == ix;

        let page = suggestions.iter().enumerate().fold(
            String::new(),
            |mut output, (ix, suggestion)| {
                if is_selected(ix) {
                    output += "  ";
                    output += &format!(
                        "{}{}",
                        color::Fg(color::Black),
                        color::Bg(color::Rgb(156, 113, 243))
                    );
                    output += suggestion;
                    output += &format!(
                        "{}{}",
                        color::Fg(color::Reset),
                        color::Bg(color::Reset)
                    );
                } else {
                    output += "  ";
                    output += suggestion;
                }

                output
            },
        );

        page
    }

    pub fn page_ix_message_space(ix: usize) -> usize {
        // The length of message `(nnn)` is 2 plus length of `ix` in decimal
        // notation.

        let mut n_digits = 1;
        let mut ten_to_power = 10;

        loop {
            if ix < ten_to_power {
                return n_digits + 2;
            } else {
                n_digits += 1;
                ten_to_power *= 10;
            }
        }
    }
}

/*
fn main() -> Result<()> {
    let cursor: Vec<String> = vec![
        "mine".into(),
        "studia".into(),
        "analiza-danych-pomiarowych".into(),
    ];
    let query = "cw";
    let suggestions = vec![
        "cw-asdfasdfasdfasdf-1",
        "cw-asdfasdfasdfasdf-2",
        "cw-asdfasdfasdfasdf-3",
        "cw-asdfasdfasdfasdf-4",
        "cw-asdfasdfasdfasdf-5",
        "cw-asdfasdfasdfasdf-6",
        "cw-asdfasdfasdfasdf-7",
        "cw-asdfasdfasdfasdf-8",
        "cw-asdfasdfasdfasdf-9",
        "cw-asdfasdfasdfasdf-10",
        "cw-asdfasdfasdfasdf-11",
        "cw-asdfasdfasdfasdf-12",
        "cw-asdfasdfasdfasdf-13",
        "cw-asdfasdfasdfasdf-14",
        "cw-asdfasdfasdfasdf-15",
        "cw-asdfasdfasdfasdf-16",
        "cw-asdfasdfasdfasdf-17",
        "cw-asdfasdfasdfasdf-18",
        "cw-asdfasdfasdfasdf-19",
        "cw-asdfasdfasdfasdf-20",
        "cw-asdfasdfasdfasdf-21",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();

    let mut ui = UI::new(&cursor, query, &suggestions)?;
    ui.display()?;

    for (event, bytes) in stdin().events_and_raw().map(|val| val.unwrap()) {
        match event {
            // `Ctrl + h` and `Ctrl + l`.
            Event::Key(Key::Ctrl('h')) => ui.previous_suggestion(),
            Event::Key(Key::Ctrl('l')) => ui.next_suggestion(),

            // `Tab` and `Shift + Tab`.
            Event::Key(Key::BackTab) => ui.previous_suggestion(),
            Event::Key(Key::Char('\t')) => ui.next_suggestion(),

            // `Ctrl + j` and `Ctrl + k`.
            Event::Key(Key::Ctrl('k')) => ui.previous_page(),
            Event::Key(_) if bytes == sequence::CTRL_J => ui.next_page(),

            // `Enter`.
            Event::Key(_) if bytes == sequence::ENTER => {
                ui.finalize()?;

                return Ok(());
            }

            // `Ctrl + c`.
            Event::Key(Key::Ctrl('c')) => {
                ui.finalize()?;

                return Ok(());
            }
            _ => {},
        }

        ui.display()?;
    }

    Ok(())
}
*/
