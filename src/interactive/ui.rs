use std::io::{stdin, stdout, Result, Stdout, Write};

use termion::{
    clear,
    color,
    cursor::{self, DetectCursorPos, Goto},
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};

// Palette:
// https://coolors.co/9c71f3-47f0a7-cca6e8-8380b6-111d4a

use unicode_width::*;

pub struct UI {
    stdout: RawTerminal<Stdout>,
}

impl UI {
    pub fn new() -> Result<Self> {
        let mut stdout = stdout().into_raw_mode()?;
        // Make room for input and results.
        write!(stdout, "\n")?;
        stdout.flush()?;
        write!(stdout, "{}", cursor::Up(1))?;
        stdout.flush()?;

        Ok(Self { stdout })
    }

    pub fn display(
        &mut self,
        cursor: &[String],
        query: &str,
        suggestions: &[String],
    ) -> Result<()> {
        // Assuming cursor is at the original input line, not necessarily at the
        // first char.
        self.print_input(cursor, query)?;
        self.print_suggestions(suggestions)?;

        Ok(())
    }

    fn compose_cursor(cursor: &[String]) -> String {
        let start_ix = cursor.len().saturating_sub(2);
        let end_ix = cursor.len();
        let prefix = if start_ix == 0 { "" } else { ".../" }.to_string();
        let cursor = cursor[start_ix..end_ix]
            .iter()
            .fold(String::new(), |cursor, component| cursor + component + "/");

        prefix + &cursor
    }

    // TODO: Delegate it to a separate type.
    fn print_input(&mut self, cursor: &[String], query: &str) -> Result<()> {
        let cursor = Self::compose_cursor(cursor);
        let (_, current_line) = self.stdout.cursor_pos()?;

        write!(
            self.stdout,
            "{clear}{goto}{cursor_fg}{cursor}{query_fg}{query}{reset_fg}{reset_bg}",
            clear = clear::CurrentLine,
            goto = cursor::Goto(1, current_line),
            cursor_fg = color::Fg(color::AnsiValue::grayscale(16)),
            cursor = cursor,
            query_fg = color::Fg(color::Rgb(156, 113, 243)),
            query = query,
            reset_fg = color::Fg(color::Reset),
            reset_bg = color::Bg(color::Reset),
        )?;
        self.stdout.flush()?;

        Ok(())
    }

    // TODO: Delegate it to a separate type.
    fn print_suggestions(&mut self, suggestions: &[String]) -> Result<()> {
        let (width, _) = termion::terminal_size()?;

        // Calculate the max space of message "(nnn/NNN)".
        let page_ix_space = length_in_decimal(suggestions.len());
        let page_info_space = 3 + 2 * page_ix_space;
        let separator_space = 1;
        let suggestions_space =
            width as usize - (page_info_space + separator_space);

        let selected = 1;

        let page_ix = 2;
        let n_pages = 10;

        // TODO: Build this on `init_state` or smth, not before every print.
        let mut suggestions_message = String::new();
        let mut new_suggestions_message = String::new();
        // TODO: Handle the first suggestion separately.
        //   - No space at the beginning.
        //   - If the suggestion's length exceeds the available space, it must be formatted differently.
        for (i, suggestion) in suggestions.iter().enumerate() {
            new_suggestions_message += " ";
            new_suggestions_message += suggestion;

            if new_suggestions_message.width() > suggestions_space as usize {
                break;
            } else {
                if i == selected {
                    suggestions_message += "  ";
                    suggestions_message += &format!(
                        "{}{}",
                        color::Fg(color::Black),
                        color::Bg(color::Rgb(156, 113, 243))
                    );
                    suggestions_message += suggestion;
                    suggestions_message += &format!(
                        "{}{}",
                        color::Fg(color::Reset),
                        color::Bg(color::Reset)
                    );
                } else {
                    suggestions_message += "  ";
                    suggestions_message += suggestion;
                }
            }
        }

        // TODO: Implement it with `let cursor_pos = CursorPos::new();` and
        // `drop(cursor_pos);`. Save cursor location.
        write!(self.stdout, "{}{}", cursor::Save, cursor::Down(1))?;
        self.stdout.flush()?;
        let current_line = self.stdout.cursor_pos()?.1;
        write!(
            self.stdout,
            "{}{}{}{}",
            cursor::Goto(page_info_space as u16 + 2, current_line),
            clear::CurrentLine,
            suggestions_message,
            cursor::Restore,
        )?;
        write!(
            self.stdout,
            "{goto}{info_fg}({page_ix:width$}/{n_pages:width$}){reset_fg}{reset_bg}",
            goto = cursor::Goto(1, current_line),
            info_fg = color::Fg(color::Rgb(71, 240, 167)),
            page_ix = page_ix,
            n_pages = n_pages,
            width = page_ix_space,
            reset_fg = color::Fg(color::Reset),
            reset_bg = color::Bg(color::Reset),
        )?;
        // Restore cursor location.
        write!(self.stdout, "{}", cursor::Restore)?;

        self.stdout.flush()?;

        Ok(())
    }
}

fn length_in_decimal(n: usize) -> usize {
    let mut power = 1;
    let mut ten_to_power = 10;

    loop {
        if n < ten_to_power {
            return power;
        } else {
            power += 1;
            ten_to_power *= 10;
        }
    }
}

impl Drop for UI {
    fn drop(&mut self) {
        let current_line = match self.stdout.cursor_pos() {
            Ok((x, y)) => y,
            Err(_) => return,
        };
        let res = write!(
            self.stdout,
            "{}{}",
            cursor::Goto(1, current_line),
            clear::AfterCursor,
        );
        match res {
            Ok(()) => {}
            Err(_) => return,
        }
        let res = self.stdout.flush();
        match res {
            Ok(()) => {}
            Err(_) => return,
        }
    }
}

fn main() -> Result<()> {
    let mut ui = UI::new()?;
    let cursor = vec![
        "mine".into(),
        "studia".into(),
        "analiza-danych-pomiarowych".into(),
    ];
    let query = "cw".into();
    let suggestions =
        vec!["cw-1".into(), "cw-2".into(), "cw-3".into(), "cw-4".into()];

    ui.display(&cursor, query, &suggestions)?;

    TermRead::read_line(&mut stdin()).unwrap();

    Ok(())
}
