use std::{io::Write, path::PathBuf};

use termion::{
    clear,
    color,
    cursor::{self, DetectCursorPos},
};
use unicode_width::*;

use crate::{
    error::{Error, Result},
    utils::as_path,
};

const SUGGESTIONS_SEPARATOR: &str = "  ";

// Palette:
// https://coolors.co/9c71f3-47f0a7-cca6e8-8380b6-111d4a

// TODO: Handle file names with length exceeding the width of the terminal.

#[derive(Debug)]
pub struct UI<'a, W>
where
    W: Write,
{
    stdout: &'a mut W,
    input: Input,
    pages: Option<Pages>,
}

impl<'a, W> UI<'a, W>
where
    W: Write,
{
    pub fn new(
        location: PathBuf,
        input: Option<String>,
        suggestions: Vec<PathBuf>,
        stdout: &'a mut W,
    ) -> Result<Self> {
        let (width, _) = termion::terminal_size()?;

        let pages = if !suggestions.is_empty() {
            let pages = Pages::new(suggestions, width as usize)?;

            Some(pages)
        } else {
            None
        };
        let input = Input::new(location, input);

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

        // Make room for input and results.
        write!(
            self.stdout,
            "{clear}\n{clear}{up}",
            clear = clear::AfterCursor,
            up = cursor::Up(1),
        )?;
        self.stdout.flush()?;

        write!(self.stdout, "\r{}", clear::AfterCursor)?;

        self.input.display(&mut self.stdout)?;
        let cursor_pos = Cursor::save(&mut self.stdout)?;
        write!(self.stdout, "{}", cursor::Down(1))?;

        if let Some(pages) = &self.pages {
            pages.display(&mut self.stdout)?;
        }

        cursor_pos.restore(&mut self.stdout)?;

        Ok(())
    }

    pub fn next_suggestion(&mut self) {
        if let Some(ref mut pages) = &mut self.pages {
            pages.next_suggestion();
        }
    }

    pub fn previous_suggestion(&mut self) {
        if let Some(ref mut pages) = &mut self.pages {
            pages.previous_suggestion();
        }
    }

    pub fn next_page(&mut self) {
        if let Some(ref mut pages) = &mut self.pages {
            pages.next_page();
        }
    }

    pub fn previous_page(&mut self) {
        if let Some(ref mut pages) = &mut self.pages {
            pages.previous_page();
        }
    }

    pub fn get_selected_suggestion(&self) -> Option<PathBuf> {
        self.pages
            .as_ref()
            .and_then(|pages| pages.get_selected_suggestion())
    }

    pub fn clear(&mut self) -> Result<()> {
        write!(self.stdout, "\r{}", clear::AfterCursor,)?;
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

        write!(
            stdout,
            "\r{clear}{location_fg}{location}",
            clear = clear::CurrentLine,
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

#[derive(Debug, Clone)]
struct Suggestion {
    path: PathBuf,
    file_name: String,
}

#[derive(Copy, Clone, Debug)]
struct Page {
    start_ix: usize,
}

#[derive(Debug)]
struct Pages {
    pages: Vec<Page>,
    suggestions: Vec<Suggestion>,
    suggestion_ix: usize,
    page_ix: usize,
}

impl Pages {
    fn new(suggestions: Vec<PathBuf>, width: usize) -> Result<Self> {
        let suggestions = Self::prepare_suggestions(suggestions);
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

    fn prepare_suggestions(suggestions: Vec<PathBuf>) -> Vec<Suggestion> {
        suggestions
            .into_iter()
            .filter_map(|path| {
                let file_name = path.file_name().map(|file_name| {
                    file_name.to_os_string().to_string_lossy().to_string()
                });

                file_name.map(|file_name| Suggestion { path, file_name })
            })
            .collect()
    }

    fn get_page_suggestions<'s>(
        page_ix: usize,
        pages: &[Page],
        suggestions: &'s [Suggestion],
    ) -> Result<&'s [Suggestion]> {
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

    fn build_pages(
        suggestions: &[Suggestion],
        space: usize,
    ) -> Result<Vec<Page>> {
        if suggestions.is_empty() {
            return Err(dev_err!("empty suggestions"));
        }

        let mut page_width = 0;
        let mut pages = vec![Page { start_ix: 0 }];
        let mut page_space = space - utils::page_ix_message_space(pages.len());

        for (i, Suggestion { file_name, .. }) in suggestions.iter().enumerate()
        {
            let delta_width = SUGGESTIONS_SEPARATOR.width() + file_name.width();
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
        suggestions: &[Suggestion],
        pages: &[Page],
    ) -> (usize, usize) {
        let len = suggestions.len();
        let suggestion_ix = suggestion_ix % len;

        let page_ix = Self::find_selected_page(suggestion_ix, pages);

        (suggestion_ix, page_ix)
    }

    fn next_suggestion(&mut self) {
        let val = Self::selection_from_suggestion(
            self.suggestion_ix + 1,
            &self.suggestions,
            &self.pages,
        );
        self.suggestion_ix = val.0;
        self.page_ix = val.1;
    }

    fn previous_suggestion(&mut self) {
        let val = Self::selection_from_suggestion(
            self.suggestion_ix + self.suggestions.len() - 1,
            &self.suggestions,
            &self.pages,
        );
        self.suggestion_ix = val.0;
        self.page_ix = val.1;
    }

    fn next_page(&mut self) {
        let val = Self::selection_from_page(self.page_ix + 1, &self.pages);
        self.suggestion_ix = val.0;
        self.page_ix = val.1;
    }

    fn previous_page(&mut self) {
        let val = Self::selection_from_page(
            self.page_ix + self.pages.len() - 1,
            &self.pages,
        );
        self.suggestion_ix = val.0;
        self.page_ix = val.1;
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

    fn get_selected_suggestion(&self) -> Option<PathBuf> {
        self.suggestions
            .get(self.suggestion_ix)
            .map(|Suggestion { path, .. }| path.clone())
    }
}

mod utils {
    use super::*;

    pub(super) fn compose_page(
        suggestions: &[Suggestion],
        selected_ix: usize,
    ) -> String {
        let is_selected = |ix| selected_ix == ix;

        let page = suggestions.iter().enumerate().fold(
            String::new(),
            |mut output, (ix, Suggestion { file_name, .. })| {
                if is_selected(ix) {
                    output += "  ";
                    output += &format!(
                        "{}{}",
                        color::Fg(color::Black),
                        color::Bg(color::Rgb(156, 113, 243))
                    );
                    output += file_name;
                    output += &format!(
                        "{}{}",
                        color::Fg(color::Reset),
                        color::Bg(color::Reset)
                    );
                } else {
                    output += "  ";
                    output += file_name;
                }

                output
            },
        );

        page
    }

    pub(super) fn page_ix_message_space(ix: usize) -> usize {
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
        let suggestions =
            vec!["aa".into(), "bb".into(), "cc".into(), "dd".into()];
        let mut pages = Pages::new(suggestions, 15).unwrap();

        pages.next_suggestion();
        assert_eq!(pages.get_selected_suggestion().unwrap(), as_path("bb"));

        pages.previous_suggestion();
        assert_eq!(pages.get_selected_suggestion().unwrap(), as_path("aa"));

        pages.next_page();
        assert_eq!(pages.get_selected_suggestion().unwrap(), as_path("dd"));

        pages.next_page();
        assert_eq!(pages.get_selected_suggestion().unwrap(), as_path("aa"));
    }
}
