mod cursor;
mod dir;
mod mode;
mod normal;
mod write_path;
use crossterm::{
    cursor::{SetCursorStyle, Show},
    terminal::{disable_raw_mode, enable_raw_mode},
    terminal::{Clear, ClearType},
};
use cursor::Cursor;
use dir::Dir;
use mode::Mode;
use normal::Normal;
use std::{
    env::args,
    fs::File,
    io::{stdout, BufReader, Read, Write},
    thread::sleep,
    time::Duration,
};
use write_path::WritePath;

fn main() {
    enable_raw_mode().unwrap();
    let mut args = args();
    let mut dir = Dir::new();
    let mut buffer = Vec::<String>::new();
    match args.nth(1) {
        Some(s) => {
            let file = File::open(&s).unwrap();
            let mut reader = BufReader::new(file);
            let mut b = String::new();
            reader.read_to_string(&mut b).unwrap();
            buffer = b.lines().map(|s| s.to_string()).collect::<Vec<_>>();
            dir.path = s;
        }
        None => {
            buffer.push(String::new());
        }
    }
    let mut stdout = stdout();
    let mut mode = Mode::Normal;
    let mut normal = Normal::new(buffer);
    let mut write_path = WritePath::new();
    write!(stdout, "{}", SetCursorStyle::SteadyBar).unwrap();
    loop {
        match mode {
            Mode::Normal => {
                if normal.run(&mut stdout, &mut mode, &dir) {
                    break;
                }
            }
            Mode::WritePath => {
                write_path.run(&mut stdout, &mut mode, &mut dir);
            }
        }
        sleep(Duration::from_millis(1));
    }
    write!(stdout, "{}{}", Clear(ClearType::All), Show).unwrap();
    disable_raw_mode().unwrap();
}
