mod cursor;
mod dir;
mod mode;
mod normal;
mod write_path;
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
use termion::{
    async_stdin,
    clear::All,
    cursor::{Show, SteadyBar},
    input::TermRead,
    raw::IntoRawMode,
};
use write_path::WritePath;

fn main() {
    let mut args = args();
    let mut dir = Dir::new();
    let mut buffer = Vec::<String>::new();
    match args.nth(1) {
        Some(s) => {
            let file = File::open(&s).unwrap();
            let mut reader = BufReader::new(file);
            let mut b = String::new();
            reader.read_to_string(&mut b).unwrap();
            buffer = b.split('\n').map(|s| s.to_string()).collect::<Vec<_>>();
            dir.path = s;
        }
        None => {
            buffer.push(String::new());
        }
    }
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut stdin = async_stdin().keys();
    let mut mode = Mode::Normal;
    let mut normal = Normal::new(buffer);
    let mut write_path = WritePath::new();
    write!(stdout, "{}", SteadyBar).unwrap();
    loop {
        match mode {
            Mode::Normal => {
                if normal.run(&mut stdout, &mut stdin, &mut mode, &dir) {
                    break;
                }
            }
            Mode::WritePath => {
                write_path.run(&mut stdout, &mut stdin, &mut mode, &dir);
            }
        }
        sleep(Duration::from_millis(1));
    }
    write!(stdout, "{}{}", All, Show).unwrap();
}
