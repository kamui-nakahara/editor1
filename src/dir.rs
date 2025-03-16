use std::{
    fs::{read_dir, DirEntry},
    path::{absolute, PathBuf},
};

pub struct Dir {
    pub path: String,
    pub dir_path: PathBuf,
    pub dirs: Vec<DirEntry>,
    pub files: Vec<DirEntry>,
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
        return Self {
            path,
            dir_path,
            dirs,
            files,
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
}
