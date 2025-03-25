use crate::Cursor;
use crossterm::{
    cursor::MoveTo,
    style::{Attribute, Color, SetAttribute, SetForegroundColor},
};
use std::{
    fs::{read_dir, DirEntry},
    io::{Stdout, Write},
    path::{absolute, PathBuf},
};

const FOREGROUND: SetForegroundColor = SetForegroundColor(Color::Rgb {
    r: 135,
    g: 175,
    b: 175,
});

pub struct Dir {
    pub path: String,
    pub dir_path: PathBuf,
    pub dirs: Vec<DirEntry>,
    pub files: Vec<DirEntry>,
    pub cursor: Cursor,
}

impl Dir {
    pub fn new() -> Self {
        let path = String::new();
        let dir_path = absolute(".").unwrap();
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        for i in read_dir(&dir_path).unwrap() {
            let f = i.unwrap();
            if f.file_type().unwrap().is_dir() {
                dirs.push(f);
            } else {
                files.push(f);
            }
        }
        dirs.sort_by_key(|item| item.file_name().to_str().unwrap().to_string());
        files.sort_by_key(|item| item.file_name().to_str().unwrap().to_string());
        let cursor = Cursor::new();
        return Self {
            path,
            dir_path,
            dirs,
            files,
            cursor,
        };
    }
    pub fn set(&mut self) {
        self.dirs.clear();
        self.files.clear();
        for i in read_dir(&self.dir_path).unwrap() {
            let f = i.unwrap();
            if f.file_type().unwrap().is_dir() {
                self.dirs.push(f);
            } else {
                self.files.push(f);
            }
        }
        self.dirs
            .sort_by_key(|item| item.file_name().to_str().unwrap().to_string());
        self.files
            .sort_by_key(|item| item.file_name().to_str().unwrap().to_string());
    }
    pub fn output(&mut self, stdout: &mut Stdout, show_cursor: bool, offset: u16) {
        let dirs_len = self.dirs.len();
        if self.cursor.y == 0 && show_cursor {
            self.printdir_underline(stdout, offset, String::from(".."));
        } else {
            self.printdir_nounderline(stdout, offset, String::from(".."));
        }
        for i in 0..dirs_len {
            let dir = &self.dirs[i];
            let pathname = dir.file_name().to_str().unwrap().to_string();
            if self.cursor.y == i + 1 && show_cursor {
                self.printdir_underline(stdout, i as u16 + 1 + offset, pathname);
            } else {
                self.printdir_nounderline(stdout, i as u16 + 1 + offset, pathname);
            }
        }
        for i in 0..self.files.len() {
            let file = &self.files[i];
            let pathname = file.file_name().to_str().unwrap().to_string();
            if self.cursor.y == i + dirs_len + 1 && show_cursor {
                self.print_underline(stdout, (i + dirs_len) as u16 + 1 + offset, pathname);
            } else {
                self.print_nounderline(stdout, (i + dirs_len) as u16 + 1 + offset, pathname);
            }
        }
    }
    fn printdir_underline(&mut self, stdout: &mut Stdout, y: u16, pathname: String) {
        write!(
            stdout,
            "{}{}{}{}{}{}",
            MoveTo(0, y),
            SetAttribute(Attribute::Underlined),
            FOREGROUND,
            pathname,
            SetForegroundColor(Color::Reset),
            SetAttribute(Attribute::NoUnderline)
        )
        .unwrap();
    }
    fn printdir_nounderline(&mut self, stdout: &mut Stdout, y: u16, pathname: String) {
        write!(
            stdout,
            "{}{}{}{}",
            MoveTo(0, y),
            FOREGROUND,
            pathname,
            SetForegroundColor(Color::Reset)
        )
        .unwrap();
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
}
