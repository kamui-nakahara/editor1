use crate::{Cursor, Dir, Mode};
use std::io::{Stdout, Write};
use termion::{
    clear::All,
    color::{Fg, Reset, Rgb},
    cursor::{Goto, Show},
    event::Key,
    input::Keys,
    raw::RawTerminal,
    style::{NoUnderline, Underline},
    terminal_size, AsyncReader,
};
const FOREGROUND: Fg<Rgb> = Fg(Rgb(135, 175, 175));
pub struct WritePath {
    cursor: Cursor,
    msg: String,
    msg_color: Fg<Rgb>,
}
impl WritePath {
    pub fn new() -> Self {
        let cursor = Cursor::new();
        let msg = String::new();
        let msg_color = Fg(Rgb(255, 255, 255));
        return Self {
            cursor,
            msg,
            msg_color,
        };
    }
    pub fn run(
        &mut self,
        stdout: &mut RawTerminal<Stdout>,
        stdin: &mut Keys<AsyncReader>,
        mode: &mut Mode,
        dir: &Dir,
    ) {
        self.output(stdout, dir);
        self.input(stdout, stdin, mode, dir);
    }
    fn output(&mut self, stdout: &mut RawTerminal<Stdout>, dir: &Dir) {
        let (_width, height) = terminal_size().unwrap();
        write!(stdout, "{}", All).unwrap();
        let dirs_len = dir.dirs.len();
        write!(
            stdout,
            "{}{}[名前を付けて保存]{}",
            Goto(1, 1),
            Fg(Rgb(0, 255, 255)),
            Fg(Reset)
        )
        .unwrap();
        if self.cursor.y == 0 {
            write!(
                stdout,
                "{}{}{}..{}{}",
                Goto(1, 2),
                Underline,
                FOREGROUND,
                Fg(Reset),
                NoUnderline
            )
            .unwrap();
        } else {
            write!(stdout, "{}{}..{}", Goto(1, 2), FOREGROUND, Fg(Reset)).unwrap();
        }
        for i in 0..dirs_len {
            let dir = &dir.dirs[i];
            let pathname = dir.file_name().to_str().unwrap().to_string();
            if self.cursor.y == i + 1 {
                write!(
                    stdout,
                    "{}{}{}{}{}{}",
                    Goto(1, i as u16 + 3),
                    Underline,
                    FOREGROUND,
                    pathname,
                    Fg(Reset),
                    NoUnderline
                )
                .unwrap();
            } else {
                write!(
                    stdout,
                    "{}{}{}{}",
                    Goto(1, i as u16 + 3),
                    FOREGROUND,
                    pathname,
                    Fg(Reset)
                )
                .unwrap();
            }
        }
        for i in 0..dir.files.len() {
            let file = &dir.files[i];
            let pathname = file.file_name().to_str().unwrap().to_string();
            if self.cursor.y == i + dirs_len + 1 {
                write!(
                    stdout,
                    "{}{}{}{}",
                    Goto(1, (i + dirs_len) as u16 + 3),
                    Underline,
                    pathname,
                    NoUnderline
                )
                .unwrap();
            } else {
                write!(stdout, "{}{}", Goto(1, (i + dirs_len) as u16 + 3), pathname,).unwrap();
            }
        }
        write!(
            stdout,
            "{}{}{}{}",
            Goto(1, height - 1),
            self.msg_color,
            self.msg,
            Fg(Reset)
        )
        .unwrap();
        stdout.flush().unwrap();
    }
    fn input(
        &mut self,
        stdout: &mut RawTerminal<Stdout>,
        stdin: &mut Keys<AsyncReader>,
        mode: &mut Mode,
        dir: &Dir,
    ) {
        if let Some(Ok(key)) = stdin.next() {
            match key {
                Key::Ctrl('c') => {
                    *mode = Mode::Normal;
                    write!(stdout, "{}", Show).unwrap();
                }
                Key::Up => {
                    if 0 < self.cursor.y {
                        self.cursor.y -= 1;
                    }
                }
                Key::Down => {
                    if self.cursor.y < dir.dirs.len() + dir.files.len() {
                        self.cursor.y += 1;
                    }
                }
                Key::Char('\n') => {}
                _ => {}
            }
        }
    }
}
