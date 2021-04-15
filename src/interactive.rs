use termion::{cursor::{DetectCursorPos, Goto}, event::Key, input::TermRead, clear};
use std::io::{Write, stdout, stdin};
use termion::raw::IntoRawMode;

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let start_line = stdout.cursor_pos().unwrap().1;
    write!(stdout, "{}", Goto(0, start_line)).unwrap();
    stdout.flush().unwrap();

    let mut path = String::new();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => break,
            Key::Char(c) => path.push(c),
            Key::Backspace => {
                while let Some(c) = path.pop() {
                    if c == '/' {
                        break;
                    }
                }
            }
            _ => {},
        }
        write!(stdout, "{}{}{}", Goto(0, start_line), clear::CurrentLine, path).unwrap();

        stdout.flush().unwrap();
    }
}
