use std::{
    io::{stdin, stdout, Result, Stdout, Write},
    mem,
};

use termion::{
    clear,
    color,
    cursor::{self, DetectCursorPos},
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};

use unicode_width::*;

const SUGGESTIONS_SEPARATOR: &'static str = "  ";
const PAGE_INFO_SPACE: usize = 9; // (nnn/NNN)

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
    stdout: RawTerminal<Stdout>,
    input: Input<'a>,
    pages: Pages<'a>,
}

impl<'a> UI<'a> {
    pub fn new(
        location: &'a [String],
        query: &'a str,
        suggestions: &'a [String],
    ) -> Result<Self> {
        let mut stdout = stdout().into_raw_mode()?;

        // Make room for input and results.
        write!(stdout, "\n")?;
        stdout.flush()?;
        write!(stdout, "{}", cursor::Up(1))?;
        stdout.flush()?;
        let (width, _) = termion::terminal_size()?;

        let pages = Pages::new(suggestions, width as usize);
        let input = Input::new(location, query);

        let ui = Self {
            stdout,
            input,
            pages,
        };

        Ok(ui)
    }

    pub fn display(&mut self) -> Result<()> {
        // Assuming cursor is at the original input line, not necessarily at the
        // first char.
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
        self.pages.display(&mut self.stdout)?;
        cursor_pos.restore(&mut self.stdout)?;

        Ok(())
    }

    pub fn next_suggestion(&mut self) {
        self.pages.next_suggestion();
    }

    pub fn previous_suggestion(&mut self) {
        self.pages.previous_suggestion();
    }

    pub fn next_page(&mut self) {
        self.pages.next_page();
    }

    pub fn previous_page(&mut self) {
        self.pages.previous_page();
    }

    pub fn finalize(self) -> Result<RawTerminal<Stdout>> {
        let UI { mut stdout, .. } = self;

        let current_line = Cursor::get_line(&mut stdout)?;
        write!(
            stdout,
            "{}{}",
            cursor::Goto(1, current_line),
            clear::AfterCursor,
        )?;
        stdout.flush()?;

        Ok(stdout)
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
struct Input<'a> {
    query: &'a str,
    location: &'a [String],
}

impl<'a> Input<'a> {
    fn new(location: &'a [String], query: &'a str) -> Self {
        Self { location, query }
    }

    fn display<W>(&self, stdout: &mut W) -> Result<()>
    where
        W: Write,
    {
        let location = Self::compose_location(self.location);
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

#[derive(Copy, Clone)]
struct Page {
    start_ix: usize,
    len: usize,
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
        let is_selected = |ix| selected_ix == (self.start_ix + ix);

        let output = suggestions
            .get(self.start_ix..(self.start_ix + self.len))
            .unwrap()
            .iter()
            .enumerate()
            .fold(String::new(), |mut output, (ix, suggestion)| {
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
            });

        let current_line = Cursor::get_line(stdout)?;

        /*
        write!(
            stdout,
            "{goto}{page_info_fg}{page_info}{reset_fg}{reset_bg}",
            goto = cursor::Goto(1, current_line),
            page_info_fg = color::Fg(color::Rgb(71, 240, 167)),
            page_info = page_info,
            reset_fg = color::Fg(color::Reset),
            reset_bg = color::Bg(color::Reset),
        )?;
        */

        write!(stdout, "{}{}", cursor::Goto(1, current_line), output,)?;

        stdout.flush()?;

        Ok(())
    }
}

// TODO: Handle the first suggestion separately.
//   - No space at the beginning.
//   - If the suggestion's length exceeds the available space, it must be
//     formatted differently.

struct Pages<'a> {
    pages: Vec<Page>,
    suggestions: &'a [String],
    selection: Option<(usize, usize)>,
}

impl<'a> Pages<'a> {
    fn new(suggestions: &'a [String], width: usize) -> Self {
        let pages = Self::build_pages(suggestions, width);
        let selection = if pages.is_empty() { None } else { Some((0, 0)) };

        Self {
            pages,
            suggestions,
            selection,
        }
    }

    fn display<W>(&self, stdout: &mut W) -> Result<()>
    where
        W: Write,
    {
        match self.selection {
            Some((suggestion, page)) => {
                if let Some(page) = self.pages.get(page) {
                    page.display(suggestion, self.suggestions, stdout)?;
                }
            }
            None => {
                // No pages.
            }
        }

        Ok(())
    }

    fn build_pages(suggestions: &[String], space: usize) -> Vec<Page> {
        if suggestions.is_empty() {
            return vec![];
        }

        let mut page_width = 0;
        let mut pages = vec![];
        let mut start_ix = 0;
        let mut is_empty = true;

        for (i, suggestion) in suggestions.iter().enumerate() {
            if is_empty {
                start_ix = i;
            }

            let delta_width =
                SUGGESTIONS_SEPARATOR.width() + suggestion.width();
            let new_width = page_width + delta_width;

            if new_width > space {
                // TODO: Check if it's not off by one.
                pages.push(Page {
                    start_ix,
                    len: i - start_ix,
                });
                page_width = delta_width;
                start_ix = i;
                is_empty = false;
            } else {
                page_width = new_width;
                is_empty = false;
            }
        }

        if !is_empty {
            // Zero length case covered at the top, so subtracting is safe.
            // TODO: Check if it's not off by one.
            pages.push(Page {
                start_ix,
                len: suggestions.len() - start_ix,
            });
        }

        pages
    }

    fn next_suggestion(&mut self) {
        // TODO: Cover cases when length is 0.
        match self.selection {
            None => {
                // Selection should only be `None` if there are no pages.
            }
            Some((suggestion, _)) => {
                let new_suggestion = (suggestion + 1) % self.suggestions.len();

                let is_selected = |page: &Page| {
                    (page.start_ix <= new_suggestion)
                        && (new_suggestion < page.start_ix + page.len)
                };

                let new_selection = self
                    .pages
                    .iter()
                    .position(|page| is_selected(page))
                    .map(|new_page| (new_suggestion, new_page));

                self.selection = new_selection;
            }
        }
    }

    fn previous_suggestion(&mut self) {
        // TODO: Cover cases when length is 0.
        match self.selection {
            None => {
                // Selection should only be `None` if there are no pages.
            }
            Some((suggestion, _)) =>
                if self.suggestions.is_empty() {
                    self.selection = None;
                } else {
                    let len = self.suggestions.len();
                    let new_suggestion = (suggestion + len - 1) % len;

                    let is_selected = |page: &Page| {
                        (page.start_ix <= new_suggestion)
                            && (new_suggestion < page.start_ix + page.len)
                    };

                    let new_selection = self
                        .pages
                        .iter()
                        .position(|page| is_selected(page))
                        .map(|new_page| (new_suggestion, new_page));

                    self.selection = new_selection;
                },
        }
    }

    fn next_page(&mut self) {
        if !self.pages.is_empty() {
            if let Some((ref mut suggestion_ix, ref mut page_ix)) =
                self.selection
            {
                *page_ix = (*page_ix + 1) % self.pages.len();
                let page = self.pages.get(*page_ix).unwrap();
                *suggestion_ix = page.start_ix;
            }
        }
    }

    fn previous_page(&mut self) {
        if !self.pages.is_empty() {
            if let Some((ref mut suggestion_ix, ref mut page_ix)) =
                self.selection
            {
                let len = self.pages.len();
                *page_ix = (*page_ix + len - 1) % len;
                let page = self.pages.get(*page_ix).unwrap();
                *suggestion_ix = page.start_ix;
            }
        }
    }
}

fn main() -> Result<()> {
    let cursor = vec![
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
    TermRead::read_line(&mut stdin()).unwrap();

    ui.next_suggestion();
    ui.display()?;
    TermRead::read_line(&mut stdin()).unwrap();

    ui.previous_suggestion();
    ui.display()?;
    TermRead::read_line(&mut stdin()).unwrap();

    ui.next_page();
    ui.display()?;
    TermRead::read_line(&mut stdin()).unwrap();

    ui.previous_page();
    ui.display()?;
    TermRead::read_line(&mut stdin()).unwrap();

    ui.finalize()?;

    Ok(())
}
