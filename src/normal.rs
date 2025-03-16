use crate::{Cursor, Dir, Mode};
use crossterm::{
    cursor::{Hide, MoveTo},
    event::{read, Event, KeyCode, KeyModifiers},
    style::{Color, SetForegroundColor},
    terminal::{window_size, Clear, ClearType},
};
use std::{
    fs::File,
    io::{BufWriter, Stdout, Write},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct Normal {
    line: String,
    max: usize,
    cursor: Cursor,
    buffer: Vec<String>,
    buffer0: Vec<String>,
    diff: bool,
}

impl Normal {
    pub fn new(buffer: Vec<String>) -> Self {
        let line = String::new();
        let max = 0;
        let cursor = Cursor::new();
        let diff = false;
        return Self {
            line,
            max,
            cursor,
            buffer0: buffer.clone(),
            buffer,
            diff,
        };
    }
    pub fn run(&mut self, stdout: &mut Stdout, mode: &mut Mode, dir: &Dir) -> bool {
        self.diff = self.buffer.eq(&self.buffer0);
        self.line = self.buffer[self.cursor.y].clone();
        self.max = self.buffer.len() - 1;
        self.output(stdout, dir);
        return self.input(stdout, mode, dir);
    }
    fn output(&mut self, stdout: &mut Stdout, dir: &Dir) {
        let height = window_size().unwrap().height;
        let lines = self.buffer.len().to_string().len();
        write!(stdout, "{}", Clear(ClearType::All)).unwrap();
        for i in 0..self.buffer.len() {
            let line = self.buffer[i].clone();
            write!(
                stdout,
                "{}{}{}{}{} {}",
                MoveTo(0, i as u16),
                " ".repeat(lines - (i + 1).to_string().len()),
                SetForegroundColor(Color::Rgb {
                    r: 127,
                    g: 127,
                    b: 127
                }),
                i + 1,
                SetForegroundColor(Color::Reset),
                line
            )
            .unwrap();
        }
        let path = if dir.path.is_empty() {
            "無題"
        } else {
            &dir.path
        };
        let msg = if self.diff { "" } else { "変更済み" };
        write!(stdout, "{}[{}]{}", MoveTo(0, height - 2), path, msg).unwrap();
        write!(
            stdout,
            "{}",
            MoveTo((self.cursor.x + 1 + lines) as u16, self.cursor.y as u16)
        )
        .unwrap();
        stdout.flush().unwrap();
    }
    fn input(&mut self, stdout: &mut Stdout, mode: &mut Mode, dir: &Dir) -> bool {
        if let Ok(Event::Key(key)) = read() {
            match key.code {
                KeyCode::Backspace => {
                    self.delete();
                }
                KeyCode::Char(c) => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        match c {
                            's' => {
                                if dir.path.is_empty() {
                                    self.save_as(mode, stdout);
                                } else {
                                    self.save(&dir.path);
                                }
                            }
                            'a' => {
                                self.save_as(mode, stdout);
                            }
                            'q' => {
                                return true;
                            }
                            _ => {}
                        }
                    } else {
                        self.typing(c);
                    }
                }
                KeyCode::Up => {
                    self.up();
                }
                KeyCode::Down => {
                    self.down();
                }
                KeyCode::Left => {
                    self.left();
                }
                KeyCode::Right => {
                    self.right();
                }
                KeyCode::Enter => {
                    self.new_line();
                }
                _ => {}
            }
        }
        return false;
    }
    fn save(&mut self, path: &String) {
        let file = File::create(path).unwrap();
        let mut writer = BufWriter::new(file);
        write!(writer, "{}\n", self.buffer.join("\n")).unwrap();
        self.buffer0 = self.buffer.clone();
    }
    fn save_as(&mut self, mode: &mut Mode, stdout: &mut Stdout) {
        write!(stdout, "{}", Hide).unwrap();
        *mode = Mode::WritePath;
    }
    fn delete(&mut self) {
        let mut w = String::new();
        let mut s = String::new();
        for c in self.line.chars() {
            w.push(c);
            if w.width() >= self.cursor.x {
                s = String::from(c);
                break;
            }
        }
        let l = w.len();
        if 0 < self.cursor.x {
            let line1 = &self.line[0..l - s.len()];
            let line2 = &self.line[l..];
            self.buffer[self.cursor.y] = format!("{}{}", line1, line2);
            self.cursor.x -= s.width();
        } else if 0 < self.cursor.y {
            let mut buffer1 = self.buffer[0..self.cursor.y].to_vec();
            let buffer2 = self.buffer[self.cursor.y].clone();
            let buffer3 = self.buffer[self.cursor.y + 1..].to_vec();
            let l = buffer1[self.cursor.y - 1].width();
            buffer1[self.cursor.y - 1].push_str(&buffer2);
            buffer1.extend(buffer3);
            self.buffer = buffer1;
            self.cursor.y -= 1;
            self.cursor.x = l;
        }
    }
    fn new_line(&mut self) {
        let mut w = String::new();
        for c in self.line.chars() {
            w.push(c);
            if w.width() >= self.cursor.x {
                break;
            }
        }
        let l = w.len();
        let line1 = &self.line[0..l];
        let line2 = &self.line[l..];
        self.buffer[self.cursor.y] = String::from(line1);
        let mut buffer1 = self.buffer[0..self.cursor.y + 1].to_vec();
        let buffer2 = self.buffer[self.cursor.y + 1..].to_vec();
        buffer1.push(String::from(line2));
        buffer1.extend(buffer2);
        self.buffer = buffer1;
        self.cursor.y += 1;
        self.cursor.x = 0;
    }
    fn typing(&mut self, c: char) {
        if 0 < self.cursor.x {
            let mut w = String::new();
            for c in self.line.chars() {
                w.push(c);
                if w.width() >= self.cursor.x {
                    break;
                }
            }
            let l = w.len();
            let line1 = &self.line[0..l];
            let line2 = &self.line[l..];
            self.buffer[self.cursor.y] = format!("{}{}{}", line1, c, line2);
        } else {
            self.buffer[self.cursor.y] = format!("{}{}", c, self.line);
        }
        self.cursor.x += UnicodeWidthChar::width(c).unwrap();
    }
    fn up(&mut self) {
        if 0 < self.cursor.y {
            self.cursor.y -= 1;
            let str = &*self.buffer[self.cursor.y].clone();
            let size = UnicodeWidthStr::width(str);
            if size == 0 || size - 1 < self.cursor.x {
                self.cursor.x = size;
            }
        } else {
            self.cursor.x = 0;
        }
    }
    fn down(&mut self) {
        if self.cursor.y < self.max {
            self.cursor.y += 1;
            let str = &*self.buffer[self.cursor.y].clone();
            let size = UnicodeWidthStr::width(str);
            if size == 0 || size - 1 < self.cursor.x {
                self.cursor.x = size;
            }
        } else {
            self.cursor.x = UnicodeWidthStr::width(&*self.line);
        }
    }
    fn left(&mut self) {
        if 0 < self.cursor.x {
            let mut w = 0;
            let mut s = 0;
            for c in self.line.chars() {
                s = c.width().unwrap();
                w += s;
                if w >= self.cursor.x {
                    break;
                }
            }
            self.cursor.x -= s;
        } else if 0 < self.cursor.y {
            self.cursor.y -= 1;
            self.cursor.x = self.buffer[self.cursor.y].width();
        }
    }
    fn right(&mut self) {
        if self.cursor.x < UnicodeWidthStr::width(&*self.line) {
            let mut w = 0;
            let mut s = 0;
            for c in self.line.chars() {
                s = c.width().unwrap();
                w += s;
                if w > self.cursor.x {
                    break;
                }
            }
            self.cursor.x += s;
        } else if self.cursor.y < self.max {
            self.cursor.y += 1;
            self.cursor.x = 0;
        }
    }
}
