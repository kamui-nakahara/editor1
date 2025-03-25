use crate::{Dir, Mode};
use crossterm::{
    cursor::{MoveTo, Show},
    event::{read, Event, KeyCode, KeyModifiers},
    style::{Color, SetForegroundColor},
    terminal::{window_size, Clear, ClearType},
};
use std::io::{Stdout, Write};

pub struct Open {}

impl Open {
    pub fn new() -> Self {
        return Self {};
    }
    pub fn run(&mut self, stdout: &mut Stdout, mode: &mut Mode, dir: &mut Dir) -> String {
        self.output(stdout, dir);
        return self.input(stdout, mode, dir);
    }
    fn output(&mut self, stdout: &mut Stdout, dir: &mut Dir) {
        let size = window_size().unwrap();
        let height = size.rows;
        write!(stdout, "{}", Clear(ClearType::All)).unwrap();
        write!(
            stdout,
            "{}{}[ファイルを開く]{}",
            MoveTo(0, 0),
            SetForegroundColor(Color::Rgb {
                r: 0,
                g: 255,
                b: 255
            }),
            SetForegroundColor(Color::Reset)
        )
        .unwrap();
        dir.output(stdout, true, 1);
        let path = dir.dir_path.to_str().unwrap();
        write!(stdout, "{}{}", MoveTo(0, height - 1), path).unwrap();
        stdout.flush().unwrap();
    }
    fn input(&mut self, stdout: &mut Stdout, mode: &mut Mode, dir: &mut Dir) -> String {
        let mut path = String::new();
        if let Ok(Event::Key(key)) = read() {
            match key.code {
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                    self.back(stdout, mode)
                }
                KeyCode::Enter => path = self.select(stdout, mode, dir),
                KeyCode::Up => {
                    if 0 < dir.cursor.y {
                        dir.cursor.y -= 1;
                    }
                }
                KeyCode::Down => {
                    if dir.cursor.y < dir.dirs.len() + dir.files.len() {
                        dir.cursor.y += 1;
                    }
                }
                _ => {}
            }
        }
        return path;
    }
    fn set(&mut self, dir: &mut Dir) {
        dir.set();
        dir.cursor.y = 0;
    }
    fn select(&mut self, stdout: &mut Stdout, mode: &mut Mode, dir: &mut Dir) -> String {
        let l = dir.dirs.len();
        let mut path = String::new();
        if 0 == dir.cursor.y {
            if let Some(p) = dir.dir_path.parent() {
                dir.dir_path = p.to_path_buf();
                self.set(dir);
            }
        } else if dir.cursor.y - 1 < l {
            dir.dir_path = dir.dir_path.join(dir.dirs[dir.cursor.y - 1].path());
            self.set(dir);
        } else {
            let p = dir.files[dir.cursor.y - 1 - l].path();
            path = dir.dir_path.join(p).to_str().unwrap().to_string();
            self.back(stdout, mode);
        }
        return path;
    }
    fn back(&mut self, stdout: &mut Stdout, mode: &mut Mode) {
        *mode = Mode::Normal;
        write!(stdout, "{}", Show).unwrap();
    }
}
