mod cursor;
mod dir;
mod mode;
mod normal;
mod open;
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
use open::Open;
use std::{
    env::args,
    fs::File,
    io::{stdout, BufReader, BufWriter, Read, Write},
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
    let mut normal = Normal::new(buffer, &mut stdout, &dir);
    let mut write_path = WritePath::new();
    let mut open = Open::new();
    write!(stdout, "{}", SetCursorStyle::SteadyBar).unwrap();
    stdout.flush().unwrap();
    let mut flag: bool;
    loop {
        let mut path = String::new();
        match mode {
            Mode::Normal => {
                (path, flag) = normal.run(&mut stdout, &mut mode, &dir);
                if flag {
                    break;
                }
                if matches!(mode, Mode::WritePath) {
                    write_path.buffer = normal.buffer.clone();
                }
            }
            Mode::WritePath => {
                path = write_path.run(&mut stdout, &mut mode, &mut dir);
            }
            Mode::Open => {
                let p = open.run(&mut stdout, &mut mode, &mut dir);
                if let Ok(file) = File::open(&p) {
                    let mut reader = BufReader::new(file);
                    let mut b = String::new();
                    if let Ok(_) = reader.read_to_string(&mut b) {
                        buffer = b.lines().map(|s| s.to_string()).collect::<Vec<_>>();
                        normal.set_buffer(buffer);
                        dir.path = p;
                    }
                }
            }
        }
        if !path.is_empty() {
            if path == dir.path {
                normal.update();
            }
            if dir.path.is_empty() {
                dir.path = path.clone();
                normal.update();
            }
            save(&normal.buffer, path);
        }
        sleep(Duration::from_millis(5));
    }
    write!(stdout, "{}{}", Clear(ClearType::All), Show).unwrap();
    disable_raw_mode().unwrap();
}

fn save(buffer: &Vec<String>, path: String) {
    let file = File::create(path).unwrap();
    let mut writer = BufWriter::new(file);
    write!(writer, "{}\n", buffer.join("\n")).unwrap();
}
