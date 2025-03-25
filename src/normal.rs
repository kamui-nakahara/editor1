use crate::{Cursor, Dir, Mode};
use crossterm::{
    cursor::{Hide, MoveTo},
    event::{read, Event, KeyCode, KeyModifiers},
    style::{Color, SetForegroundColor},
    terminal::{window_size, Clear, ClearType},
};
use std::io::{Stdout, Write};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
pub struct Normal {
    line: String,
    max: usize,
    cursor: Cursor,
    pub buffer: Vec<String>,
    buffer0: Vec<String>,
    diff: bool,
    buffer_offset: usize,
    width: u16,
    height: u16,
    old_all: (u16, u16, Vec<String>, usize),
}
impl Normal {
    pub fn new(buffer: Vec<String>, stdout: &mut Stdout, dir: &Dir) -> Self {
        let line = String::new();
        let max = 0;
        let cursor = Cursor::new();
        let diff = false;
        let buffer_offset = 0;
        let width = 0;
        let height = 0;
        let old_all = (width, height, buffer.clone(), buffer_offset);
        let mut normal = Self {
            line,                    //bufferに依存
            max,                     //bufferに依存
            cursor,                  //output_cursorを実行
            buffer0: buffer.clone(), //output_msgを実行
            buffer, //変更された行に対してoutput_linesを実行し、行数が変わった場合はoutput_allを実行
            diff,   //bufferとbuffer0に依存
            buffer_offset, //output_allを実行
            width,  //output_allを実行
            height, //output_allを実行
            old_all,
        };
        normal.set_data();
        normal.output_all(stdout, dir);
        return normal;
    }
    pub fn set_buffer(&mut self, buffer: Vec<String>) {
        self.buffer = buffer;
        self.cursor = Cursor::new();
        self.update();
    }
    fn set_data(&mut self) {
        self.diff = self.buffer.eq(&self.buffer0);
        self.line = self.buffer[self.cursor.y].clone();
        self.max = self.buffer.len() - 1;
        let size = window_size().unwrap();
        self.width = size.columns;
        self.height = size.rows;
    }
    fn update_data(&mut self) {
        self.old_all = (
            self.width,
            self.height,
            self.buffer.clone(),
            self.buffer_offset,
        );
    }
    pub fn run(&mut self, stdout: &mut Stdout, mode: &mut Mode, dir: &Dir) -> (String, bool) {
        self.set_data();
        if self.old_all
            == (
                self.width,
                self.height,
                self.buffer.clone(),
                self.buffer_offset,
            )
        {
            self.output_all(stdout, dir);
        }
        let v = self.input(stdout, mode, dir);
        self.update_data();
        return v;
    }
    fn output_lines(&self, stdout: &mut Stdout, dir: &Dir, line1: usize, line2: usize) {
        let l = self.buffer.len();
        let mut x = self.cursor.x;
        let mut y = self.cursor.y;
        let lines = l.to_string().len();
        let width = self.width as usize - lines - 1;
        write!(stdout, "{}", Clear(ClearType::All)).unwrap();
        let mut ln = 0;
        let mut i = if line1 < self.buffer_offset {
            line1
        } else {
            self.buffer_offset
        };
        let end = if line2 < l { l } else { line2 };
        while i < end {
            let mut output;
            let mut line = self.buffer[i].clone();
            output = format!(
                "{}{}{}{}{} ",
                MoveTo(0, ln as u16),
                " ".repeat(lines - (i + 1).to_string().len()),
                SetForegroundColor(Color::Rgb {
                    r: 127,
                    g: 127,
                    b: 127
                }),
                i + 1,
                SetForegroundColor(Color::Reset)
            );
            while width < line.width() {
                let mut w = String::new();
                for c in line.chars() {
                    if w.width() + c.width().unwrap() > width {
                        break;
                    }
                    w.push(c);
                }
                output = format!("{}{}{}", output, MoveTo(lines as u16 + 1, ln as u16), w);
                line = line[w.len()..].to_string();
                if w.width() < x {
                    x -= w.width();
                    y += 1;
                }
                if i < self.cursor.y {
                    y += 1;
                }
                ln += 1;
            }
            output = format!("{}{}{}", output, MoveTo(lines as u16 + 1, ln as u16), line);
            ln += 1;
            i += 1;
            if ln + 1 > self.height {
                break;
            }
            write!(stdout, "{}", output).unwrap();
        }
        self.output_msg(stdout, dir);
        self.output_cursor(
            stdout,
            (x + 1 + lines) as u16,
            (y - self.buffer_offset) as u16,
        );
        stdout.flush().unwrap();
    }
    fn output_msg(&self, stdout: &mut Stdout, dir: &Dir) {
        let path = if dir.path.is_empty() {
            "無題"
        } else {
            &dir.path
        };
        let msg = if self.diff { "" } else { "変更済み" };
        write!(stdout, "{}[{}]{}", MoveTo(0, self.height - 1), path, msg).unwrap();
    }
    fn output_all(&self, stdout: &mut Stdout, dir: &Dir) {
        self.output_lines(stdout, dir, self.buffer_offset, self.buffer.len());
    }
    fn output_cursor(&self, stdout: &mut Stdout, x: u16, y: u16) {
        write!(stdout, "{}", MoveTo(x, y)).unwrap();
    }
    fn input(&mut self, stdout: &mut Stdout, mode: &mut Mode, dir: &Dir) -> (String, bool) {
        let mut flag = false;
        let mut path = String::new();
        if let Ok(Event::Key(key)) = read() {
            match key.code {
                KeyCode::Backspace => {
                    self.delete();
                }
                KeyCode::Char(c) => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        match c {
                            'o' => {
                                write!(stdout, "{}", Hide).unwrap();
                                *mode = Mode::Open;
                            }
                            's' => {
                                if dir.path.is_empty() {
                                    self.save_as(mode, stdout);
                                } else {
                                    path = dir.path.clone();
                                    self.update();
                                }
                            }
                            'a' => {
                                self.save_as(mode, stdout);
                            }
                            'q' => {
                                flag = true;
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
        return (path, flag);
    }
    pub fn update(&mut self) {
        self.buffer0 = self.buffer.clone();
    }
    fn save_as(&self, mode: &mut Mode, stdout: &mut Stdout) {
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
            self.move_up();
            self.cursor.x = l;
        }
    }
    fn new_line(&mut self) {
        if self.cursor.x == 0 {
            let mut buffer1 = self.buffer[0..self.cursor.y].to_vec();
            let buffer2 = self.buffer[self.cursor.y..].to_vec();
            buffer1.push(String::new());
            buffer1.extend(buffer2);
            self.buffer = buffer1;
            self.move_down();
        } else {
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
            self.move_down();
            self.cursor.x = 0;
        }
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
    fn move_up(&mut self) {
        self.cursor.y -= 1;
        if self.cursor.y < self.buffer_offset {
            self.buffer_offset -= 1;
        }
    }
    fn move_down(&mut self) {
        self.cursor.y += 1;
        let mut y = 0;
        let mut i = self.buffer_offset;
        let lines = self.buffer.len().to_string().len();
        let width = self.width as usize - lines - 1;
        while i < self.cursor.y {
            let mut line = self.buffer[i].clone();
            while width < line.width() {
                let mut w = String::new();
                for c in line.chars() {
                    if w.width() + c.width().unwrap() > width {
                        break;
                    }
                    w.push(c);
                }
                line = line[w.len()..].to_string();
                if i < self.cursor.y {
                    y += 1;
                }
            }
            y += 1;
            i += 1;
        }
        let mut len = 0;
        let mut line = self.buffer[self.cursor.y].clone();
        while width < line.width() {
            let mut w = String::new();
            for c in line.chars() {
                if w.width() + c.width().unwrap() > width {
                    break;
                }
                w.push(c);
            }
            line = line[w.len()..].to_string();
            len += 1;
        }
        len += 1;
        y = y + len - 1;
        if y >= self.height - 1 {
            self.buffer_offset += 1;
        }
    }
    fn up(&mut self) {
        if 0 < self.cursor.y {
            self.move_up();
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
            self.move_down();
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
            self.move_up();
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
            self.move_down();
            self.cursor.x = 0;
        }
    }
}
