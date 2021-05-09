use std::{
    io::{Stdout, Write},
    path::PathBuf,
};

use termion::{
    clear,
    color,
    cursor::{self, DetectCursorPos},
    raw::RawTerminal,
};

use unicode_width::*;

use crate::{
    error::{Error, Result},
    utils::as_path,
};

/*
#[path = "../error.rs"]
mod error;
use error::{Error, Result};

#[path = "../utils.rs"]
#[macro_use] mod _utils;
use _utils::as_path;
*/

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

#[derive(Debug)]
pub struct UIState {
    pub suggestions: Vec<String>, /* TODO: Possible problems with converting
                                   * OsString to String? */
    pub input: Option<String>,
    pub location: PathBuf,
}

#[derive(Debug)]
pub struct UI<'a, W> where W: Write {
    stdout: &'a mut W,
    input: Input,
    pages: Option<Pages>,
}

impl<'a, W> UI<'a, W> where W: Write {
    pub fn new(
        stdout: &'a mut W,
        UIState {
            location,
            input,
            suggestions,
        }: UIState,
    ) -> Result<(Self, Option<usize>)> {
        let (width, _) = termion::terminal_size()?;

        let pages = if !suggestions.is_empty() {
            let pages = Pages::new(suggestions, width as usize)?;

            Some(pages)
        } else {
            None
        };
        let suggestion_ix = pages.as_ref().map(|pages| pages.suggestion_ix);
        let input = Input::new(location, input);

        let ui = Self {
            stdout,
            input,
            pages,
        };

        Ok((ui, suggestion_ix))
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

        if let Some(pages) = &self.pages {
            pages.display(&mut self.stdout)?;
        }

        cursor_pos.restore(&mut self.stdout)?;

        Ok(())
    }

    pub fn next_suggestion(&mut self) -> Option<usize> {
        self.pages.as_mut().map(|pages| pages.next_suggestion())
    }

    pub fn previous_suggestion(&mut self) -> Option<usize> {
        self.pages.as_mut().map(|pages| pages.previous_suggestion())
    }

    pub fn next_page(&mut self) -> Option<usize> {
        self.pages.as_mut().map(|pages| pages.next_page())
    }

    pub fn previous_page(&mut self) -> Option<usize> {
        self.pages.as_mut().map(|pages| pages.previous_page())
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

    pub fn take(self) -> &'a mut W {
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

#[derive(Debug)]
struct Input {
    location: PathBuf,
    query: Option<String>,
}

impl Input {
    fn new(location: PathBuf, query: Option<String>) -> Self {
        Self { location, query }
    }

    fn display<W>(&self, stdout: &mut W) -> Result<()>
    where
        W: Write,
    {
        let location = &self.location;
        let current_line = Cursor::get_line(stdout)?;

        write!(
            stdout,
            "{clear}{goto}{location_fg}{location}",
            clear = clear::CurrentLine,
            goto = cursor::Goto(1, current_line),
            location_fg = color::Fg(color::AnsiValue::grayscale(16)),
            location = location.display(),
        )?;

        if location != as_path("") {
            write!(stdout, "/")?;
        }

        if let Some(query) = &self.query {

            write!(
                stdout,
                "{query_fg}{query}",
                query_fg = color::Fg(color::Rgb(156, 113, 243)),
                query = query,
            )?;
        }
        write!(
            stdout,
            "{reset_fg}{reset_bg}",
            reset_fg = color::Fg(color::Reset),
            reset_bg = color::Bg(color::Reset),
        )?;
        stdout.flush()?;

        Ok(())
    }
}

// TODO: Handle the first suggestion separately.
//   - No space at the beginning.
//   - If the suggestion's length exceeds the available space, it must be
//     formatted differently.

#[derive(Copy, Clone, Debug)]
struct Page {
    start_ix: usize,
}

#[derive(Debug)]
struct Pages {
    pages: Vec<Page>,
    suggestions: Vec<String>,
    suggestion_ix: usize,
    page_ix: usize,
}

impl Pages {
    fn new(suggestions: Vec<String>, width: usize) -> Result<Self> {
        let pages = Self::build_pages(&suggestions, width)?;

        let pages = Self {
            pages,
            suggestions,
            page_ix: 0,
            suggestion_ix: 0,
        };

        Ok(pages)
    }

    fn display<W>(&self, stdout: &mut W) -> Result<()>
    where
        W: Write,
    {
        if !self.pages.is_empty() {
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
        }

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

    fn build_pages(suggestions: &[String], space: usize) -> Result<Vec<Page>> {
        // TODO: Handle filenames exceeding terminal width;
        if suggestions.is_empty() {
            return Err(dev_err!("empty suggestions"));
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

        Ok(pages)
    }

    fn selection_from_page(page_ix: usize, pages: &[Page]) -> (usize, usize) {
        let len = pages.len();
        let page_ix = page_ix % len;
        let suggestion_ix = pages.get(page_ix).unwrap().start_ix;

        (suggestion_ix, page_ix)
    }

    fn selection_from_suggestion(
        suggestion_ix: usize,
        suggestions: &[String],
        pages: &[Page],
    ) -> (usize, usize) {
        let len = suggestions.len();
        let suggestion_ix = suggestion_ix % len;

        let page_ix = Self::find_selected_page(suggestion_ix, pages);

        (suggestion_ix, page_ix)
    }

    fn next_suggestion(&mut self) -> usize {
        (self.suggestion_ix, self.page_ix) = Self::selection_from_suggestion(
            self.suggestion_ix + 1,
            &self.suggestions,
            &self.pages,
        );

        self.suggestion_ix
    }

    fn previous_suggestion(&mut self) -> usize {
        (self.suggestion_ix, self.page_ix) = Self::selection_from_suggestion(
            self.suggestion_ix + self.suggestions.len() - 1,
            &self.suggestions,
            &self.pages,
        );

        self.suggestion_ix
    }

    fn next_page(&mut self) -> usize {
        (self.suggestion_ix, self.page_ix) =
            Self::selection_from_page(self.page_ix + 1, &self.pages);

        self.suggestion_ix
    }

    fn previous_page(&mut self) -> usize {
        (self.suggestion_ix, self.page_ix) = Self::selection_from_page(
            self.page_ix + self.pages.len() - 1,
            &self.pages,
        );

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

fn main() {}

#[cfg(test)]
mod test {
    use super::*;

    struct MockStdout;

    impl std::io::Write for MockStdout {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_navigation() {
        let mut stdout = MockStdout;
        let suggestions = vec!["aa".into(), "bb".into(), "cc".into(), "dd".into()];
        let mut pages = Pages::new(suggestions, 15).unwrap();

        assert_eq!(pages.next_suggestion(), 1);
        assert_eq!(pages.next_suggestion(), 2);
        assert_eq!(pages.next_page(), 3);
        assert_eq!(pages.next_page(), 0);
    }
}
