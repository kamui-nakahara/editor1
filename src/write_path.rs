use crate::{Cursor, Dir, Mode};
use crossterm::{
    cursor::{MoveTo, Show},
    event::{read, Event, KeyCode, KeyModifiers},
    style::{Attribute, Color, SetAttribute, SetForegroundColor},
    terminal::{window_size, Clear, ClearType},
};
use std::{
    io::{Stdout, Write},
    path::absolute,
};

const FOREGROUND: SetForegroundColor = SetForegroundColor(Color::Rgb {
    r: 135,
    g: 175,
    b: 175,
});

pub struct WritePath {
    cursor: Cursor,
    msg: String,
    msg_color: SetForegroundColor,
    input_mode: InputMode,
}
impl WritePath {
    pub fn new() -> Self {
        let cursor = Cursor::new();
        let msg = format!("{}", absolute(".").unwrap().to_str().unwrap());
        let msg_color = SetForegroundColor(Color::Rgb {
            r: 255,
            g: 255,
            b: 255,
        });
        let input_mode = InputMode::Select;
        return Self {
            cursor,
            msg,
            msg_color,
            input_mode,
        };
    }
    pub fn run(&mut self, stdout: &mut Stdout, mode: &mut Mode, dir: &mut Dir) {
        self.output(stdout, dir);
        self.input(stdout, mode, dir);
    }
    fn output(&mut self, stdout: &mut Stdout, dir: &Dir) {
        let height = window_size().unwrap().height;
        write!(stdout, "{}", Clear(ClearType::All)).unwrap();
        let dirs_len = dir.dirs.len();
        write!(
            stdout,
            "{}{}[名前を付けて保存]{}",
            MoveTo(0, 0),
            SetForegroundColor(Color::Rgb {
                r: 0,
                g: 255,
                b: 255
            }),
            SetForegroundColor(Color::Reset)
        )
        .unwrap();
        if self.cursor.y == 0 {
            self.print_underline(stdout, 1, String::from(".."));
        } else {
            self.print_nounderline(stdout, 1, String::from(".."));
        }
        for i in 0..dirs_len {
            let dir = &dir.dirs[i];
            let pathname = dir.file_name().to_str().unwrap().to_string();
            if self.cursor.y == i + 1 {
                self.print_underline(stdout, i as u16 + 2, pathname);
            } else {
                self.print_nounderline(stdout, i as u16 + 2, pathname);
            }
        }
        for i in 0..dir.files.len() {
            let file = &dir.files[i];
            let pathname = file.file_name().to_str().unwrap().to_string();
            if self.cursor.y == i + dirs_len + 1 {
                self.print_underline(stdout, (i + dirs_len) as u16 + 2, pathname);
            } else {
                self.print_nounderline(stdout, (i + dirs_len) as u16 + 2, pathname);
            }
        }
        write!(
            stdout,
            "{}{}{}{}",
            MoveTo(0, height - 2),
            self.msg_color,
            self.msg,
            SetForegroundColor(Color::Reset)
        )
        .unwrap();
        stdout.flush().unwrap();
    }
    fn print_underline(&mut self, stdout: &mut Stdout, y: u16, pathname: String) {
        write!(
            stdout,
            "{}{}{}{}",
            MoveTo(0, y),
            SetAttribute(Attribute::Underlined),
            pathname,
            SetAttribute(Attribute::NoUnderline)
        )
        .unwrap();
    }
    fn print_nounderline(&mut self, stdout: &mut Stdout, y: u16, pathname: String) {
        write!(stdout, "{}{}", MoveTo(0, y), pathname,).unwrap();
    }
    fn input(&mut self, stdout: &mut Stdout, mode: &mut Mode, dir: &mut Dir) {
        if let Ok(Event::Key(key)) = read() {
            match key.code {
                KeyCode::Tab => {
                    if matches!(self.input_mode, InputMode::Select) {
                        self.input_mode = InputMode::Write;
                    } else {
                        self.input_mode = InputMode::Select;
                    }
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Select;
                }
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                    *mode = Mode::Normal;
                    write!(stdout, "{}", Show).unwrap();
                }
                KeyCode::Up if matches!(self.input_mode, InputMode::Select) => {
                    if 0 < self.cursor.y {
                        self.cursor.y -= 1;
                    }
                }
                KeyCode::Down if matches!(self.input_mode, InputMode::Select) => {
                    if self.cursor.y < dir.dirs.len() + dir.files.len() {
                        self.cursor.y += 1;
                    }
                }
                KeyCode::Left if matches!(self.input_mode, InputMode::Write) => {}
                KeyCode::Right if matches!(self.input_mode, InputMode::Write) => {}
                KeyCode::Enter => {
                    if matches!(self.input_mode, InputMode::Select) {
                        self.select(dir);
                    } else {
                        self.write();
                    }
                }
                _ => {}
            }
        }
    }
    fn write(&mut self) {}
    fn select(&mut self, dir: &mut Dir) {
        let l = dir.dirs.len();
        if 0 == self.cursor.y {
            if let Some(p) = dir.dir_path.parent() {
                dir.dir_path = p.to_path_buf();
                self.set(dir);
            }
        } else if self.cursor.y - 1 < l {
            dir.dir_path = dir.dir_path.join(dir.dirs[self.cursor.y - 1].path());
            self.set(dir);
        } else {
            self.msg = format!(
                "{} {}",
                dir.dir_path
                    .join(dir.files[self.cursor.y - 1 - l].path())
                    .to_str()
                    .unwrap(),
                "このファイルは既に存在します。上書きしますか？(y/n)"
            );
            self.msg_color = SetForegroundColor(Color::Rgb {
                r: 215,
                g: 135,
                b: 95,
            });
        }
    }
    fn set(&mut self, dir: &mut Dir) {
        dir.set();
        self.msg = format!("{}", dir.dir_path.to_str().unwrap());
        self.cursor.y = 0;
        self.msg_color = SetForegroundColor(Color::Rgb {
            r: 255,
            g: 255,
            b: 255,
        });
    }
}

enum InputMode {
    Select,
    Write,
    Check,
}
