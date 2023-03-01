use alloc::{collections::BTreeMap, string::String, vec, vec::Vec};
use spin::{Lazy, Mutex};

#[derive(Debug)]
pub enum FileError {
    NotFoundFile,
    NotFoundDir,
    AlreadyFile,
    AlreadyDir,
}

#[derive(Debug, Clone)]
pub struct File {
    name: String,
    content: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Path {
    inner: Vec<String>,
}

use core::fmt::Display;

impl Display for Path {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl Path {
    pub fn from_str(s: &str) -> Self {
        Self {
            inner: vec![s.into()],
        }
    }

    pub fn pop_parent(&mut self) -> Option<String> {
        assert!(self.inner.len() > 1);
        self.inner.pop()
    }

    pub fn have_parent(&self) -> bool {
        self.inner.len() > 1
    }

    pub fn file_name(&self) -> &String {
        assert!(self.inner.len() == 1);
        &self.inner[0]
    }
}

impl File {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            content: Vec::new(),
        }
    }

    pub fn with_content(name: &str, content: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            content,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn content(&self) -> &Vec<u8> {
        &self.content
    }
}

pub struct Directory {
    dirs: BTreeMap<String, Directory>,
    files: BTreeMap<String, File>,
}

impl Directory {
    fn new() -> Self {
        Self {
            dirs: BTreeMap::new(),
            files: BTreeMap::new(),
        }
    }

    fn create_file(&mut self, path: &mut Path) -> Result<(), FileError> {
        if path.have_parent() {
            let dir = path.pop_parent().unwrap();
            if let Some(dir) = self.dirs.get_mut(&dir) {
                dir.create_file(path)
            } else {
                Err(FileError::NotFoundDir)
            }
        } else {
            if let None = self.files.get(path.file_name()) {
                self.files
                    .insert(path.file_name().clone(), File::new(&path.file_name()));
                Ok(())
            } else {
                Err(FileError::AlreadyFile)
            }
        }
    }

    fn find_file(&self, path: &mut Path) -> Result<&File, FileError> {
        if path.have_parent() {
            let dir = path.pop_parent().unwrap();
            if let Some(dir) = self.dirs.get(&dir) {
                dir.find_file(path)
            } else {
                Err(FileError::NotFoundDir)
            }
        } else {
            if let Some(file) = self.files.get(path.file_name()) {
                Ok(file)
            } else {
                Err(FileError::AlreadyFile)
            }
        }
    }

    fn find_files(&self, path: &mut Path) -> Result<&BTreeMap<String, File>, FileError> {
        if path.have_parent() {
            let dir = path.pop_parent().unwrap();
            if let Some(dir) = self.dirs.get(&dir) {
                dir.find_files(path)
            } else {
                Err(FileError::NotFoundDir)
            }
        } else {
            if let Some(dir) = self.files.get(path.file_name()) {
                if let Some(dir) = self.dirs.get(dir.name()) {
                    Ok(&dir.files)
                } else {
                    Err(FileError::NotFoundDir)
                }
            } else {
                Err(FileError::AlreadyFile)
            }
        }
    }
}

pub struct FileSystem {
    root_dir: Directory,
}

impl FileSystem {
    fn new() -> Self {
        Self {
            root_dir: Directory::new(),
        }
    }

    fn create_file(&mut self, path: &mut Path) -> Result<(), FileError> {
        self.root_dir.create_file(path)
    }
}

static FILE_SYSTEM: Lazy<Mutex<FileSystem>> = Lazy::new(|| Mutex::new(FileSystem::new()));

pub fn create_file(path: &mut Path) -> Result<(), FileError> {
    FILE_SYSTEM.lock().create_file(path)
}

pub fn handle_file<F: Fn(Result<&File, Path>)>(f: F, mut path: Path) {
    let c_path = path.clone();
    let file_system = FILE_SYSTEM.lock();
    let file = file_system
        .root_dir
        .find_file(&mut path)
        .map_err(|_| c_path);
    f(file);
}

pub fn handle_files<F: Fn(Result<&BTreeMap<String, File>, Path>)>(f: F, mut path: Path) {
    let c_path = path.clone();
    let file_system = FILE_SYSTEM.lock();
    let file = file_system
        .root_dir
        .find_files(&mut path)
        .map_err(|_| c_path);
    f(file);
}
