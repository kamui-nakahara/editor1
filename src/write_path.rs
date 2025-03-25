use crate::{Cursor, Dir, Mode};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyModifiers},
    style::{Color, SetForegroundColor},
    terminal::{window_size, Clear, ClearType},
};
use std::{
    io::{Stdout, Write},
    path::absolute,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct WritePath {
    cursor: Cursor,
    msg: String,
    msg_color: SetForegroundColor,
    input_mode: InputMode,
    pub buffer: Vec<String>,
    path: String,
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
        let input_mode = InputMode::Write;
        let buffer = Vec::new();
        let path = String::new();
        return Self {
            cursor,
            msg,
            msg_color,
            input_mode,
            buffer,
            path,
        };
    }
    pub fn run(&mut self, stdout: &mut Stdout, mode: &mut Mode, dir: &mut Dir) -> String {
        self.output(stdout, dir);
        return self.input(stdout, mode, dir);
    }
    fn output(&mut self, stdout: &mut Stdout, dir: &mut Dir) {
        let size = window_size().unwrap();
        let height = size.rows;
        let flag = matches!(self.input_mode, InputMode::Write);
        write!(stdout, "{}", Clear(ClearType::All)).unwrap();
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
        dir.output(stdout, !flag, 1);
        if flag {
            write!(stdout, "{}", Show).unwrap();
        } else {
            write!(stdout, "{}", Hide).unwrap();
        }
        write!(
            stdout,
            "{}{}{}{}",
            MoveTo(0, height - 1),
            self.msg_color,
            self.msg,
            SetForegroundColor(Color::Reset)
        )
        .unwrap();
        write!(
            stdout,
            "{}ファイル名を入力 > {}{}",
            MoveTo(0, height - 2),
            self.path,
            MoveTo(19 + self.cursor.x as u16, height - 2)
        )
        .unwrap();
        stdout.flush().unwrap();
    }
    fn input(&mut self, stdout: &mut Stdout, mode: &mut Mode, dir: &mut Dir) -> String {
        let mut path = String::new();
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
                    self.back(stdout, mode);
                }
                KeyCode::Up if matches!(self.input_mode, InputMode::Select) => {
                    if 0 < dir.cursor.y {
                        dir.cursor.y -= 1;
                    }
                }
                KeyCode::Down if matches!(self.input_mode, InputMode::Select) => {
                    if dir.cursor.y < dir.dirs.len() + dir.files.len() {
                        dir.cursor.y += 1;
                    }
                }
                KeyCode::Left if matches!(self.input_mode, InputMode::Write) => self.left(),
                KeyCode::Right if matches!(self.input_mode, InputMode::Write) => self.right(),
                KeyCode::Backspace if matches!(self.input_mode, InputMode::Write) => self.delete(),
                KeyCode::Enter => {
                    if matches!(self.input_mode, InputMode::Select) {
                        self.select(dir);
                    } else {
                        if !self.path.is_empty() {
                            path = self.path.clone();
                            self.back(stdout, mode);
                        }
                    }
                }
                KeyCode::Char('y') if matches!(self.input_mode, InputMode::Check) => {
                    path = dir.files[dir.cursor.y - 1 - dir.dirs.len()]
                        .path()
                        .to_str()
                        .unwrap()
                        .to_string();
                    self.back(stdout, mode);
                }
                KeyCode::Char('n') if matches!(self.input_mode, InputMode::Check) => {
                    self.input_mode = InputMode::Select;
                }
                KeyCode::Char(c) if matches!(self.input_mode, InputMode::Write) => {
                    self.typing(c);
                }
                _ => {}
            }
        }
        return path;
    }
    fn delete(&mut self) {
        let mut w = String::new();
        let mut s = String::new();
        for c in self.path.chars() {
            w.push(c);
            if w.width() >= self.cursor.x {
                s = String::from(c);
                break;
            }
        }
        let l = w.len();
        if 0 < self.cursor.x {
            let path1 = &self.path[0..l - s.len()];
            let path2 = &self.path[l..];
            self.path = format!("{}{}", path1, path2);
            self.cursor.x -= s.width();
        }
    }
    fn left(&mut self) {
        if 0 < self.cursor.x {
            let mut w = 0;
            let mut s = 0;
            for c in self.path.chars() {
                s = c.width().unwrap();
                w += s;
                if w >= self.cursor.x {
                    break;
                }
            }
            self.cursor.x -= s;
        }
    }
    fn right(&mut self) {
        if self.cursor.x < UnicodeWidthStr::width(&*self.path) {
            let mut w = 0;
            let mut s = 0;
            for c in self.path.chars() {
                s = c.width().unwrap();
                w += s;
                if w > self.cursor.x {
                    break;
                }
            }
            self.cursor.x += s;
        }
    }
    fn typing(&mut self, c: char) {
        if 0 < self.cursor.x {
            let mut w = String::new();
            for c in self.path.chars() {
                w.push(c);
                if w.width() >= self.cursor.x {
                    break;
                }
            }
            let l = w.len();
            let path1 = &self.path[0..l];
            let path2 = &self.path[l..];
            self.path = format!("{}{}{}", path1, c, path2);
        } else {
            self.path = format!("{}{}", c, self.path);
        }
        self.cursor.x += c.width().unwrap();
    }
    fn back(&mut self, stdout: &mut Stdout, mode: &mut Mode) {
        self.input_mode = InputMode::Write;
        *mode = Mode::Normal;
        write!(stdout, "{}", Show).unwrap();
    }
    fn select(&mut self, dir: &mut Dir) {
        let l = dir.dirs.len();
        if 0 == dir.cursor.y {
            if let Some(p) = dir.dir_path.parent() {
                dir.dir_path = p.to_path_buf();
                self.set(dir);
            }
        } else if dir.cursor.y - 1 < l {
            dir.dir_path = dir.dir_path.join(dir.dirs[dir.cursor.y - 1].path());
            self.set(dir);
        } else {
            self.msg = format!(
                "{} {}",
                dir.dir_path
                    .join(dir.files[dir.cursor.y - 1 - l].path())
                    .to_str()
                    .unwrap(),
                "このファイルは既に存在します。上書きしますか？(y/n)"
            );
            self.msg_color = SetForegroundColor(Color::Rgb {
                r: 215,
                g: 135,
                b: 95,
            });
            self.input_mode = InputMode::Check;
        }
    }
    fn set(&mut self, dir: &mut Dir) {
        dir.set();
        self.msg = format!("{}", dir.dir_path.to_str().unwrap());
        dir.cursor.y = 0;
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
