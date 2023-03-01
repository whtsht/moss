use crate::fs;

use alloc::string::String;
use pc_keyboard::DecodedKey;
use spin::Mutex;

use crate::print;

static _ONLY_HLT: &[u8] = include_bytes!("../onlyhlt");

static TERMINAL: Mutex<Terminal> = Mutex::new(Terminal::new());

pub fn push_key(key: DecodedKey) {
    let mut terminal = { TERMINAL.lock() };
    match key {
        DecodedKey::Unicode(character) => match character as u8 {
            8 => {
                crate::vga_buffer::backspace();
                terminal.pop();
            }
            10 => {
                print!("\n");
                terminal.run();
            }
            _ => {
                print!("{}", character);
                terminal.push(character);
            }
        },
        DecodedKey::RawKey(_) => {}
    }
}

pub fn backspace() {
    let mut terminal = TERMINAL.lock();
    terminal.buffer.pop();
}

pub struct Terminal {
    buffer: String,
}

impl Terminal {
    pub const fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn push(&mut self, c: char) {
        self.buffer.push(c);
    }

    pub fn pop(&mut self) {
        self.buffer.pop();
    }

    pub fn run(&mut self) {
        self.execute();
        self.buffer_clear();
        print!("\n>");
    }

    pub fn buffer_clear(&mut self) {
        self.buffer.clear();
    }

    pub fn execute(&mut self) {
        let mut commands = self.buffer.split(" ");
        if commands.clone().count() <= 0 {
            return;
        }

        let command = commands.next().unwrap();

        match command {
            "echo" => echo(commands),
            "touch" => touch(commands),
            "find" => find(commands),
            "onlyhlt" => exec(commands),
            _ => print!("command not found: {}", command),
        }
    }
}

fn echo<'a>(commands: impl Iterator<Item = &'a str>) {
    for args in commands {
        for c in args.chars() {
            print!("{}", c);
        }
    }
}

fn touch<'a>(mut commands: impl Iterator<Item = &'a str>) {
    if let Some(name) = commands.next() {
        fs::create_file(&mut fs::Path::from_str(name)).unwrap();
    }
}

fn find<'a>(mut commands: impl Iterator<Item = &'a str>) {
    match commands.next() {
        Some(ty) if ty == "file" => {
            if let Some(fname) = commands.next() {
                let path = fs::Path::from_str(fname);
                fs::handle_file(
                    |file| match file {
                        Ok(file) => print!("{}", file.name()),
                        Err(_) => print!("`{}`: No such file", fname),
                    },
                    path,
                )
            } else {
                print!("Unspecified file name");
            }
        }
        Some(ty) if ty == "dir" => {
            if let Some(fname) = commands.next() {
                let path = fs::Path::from_str(fname);
                fs::handle_files(
                    |files| match files {
                        Ok(files) => {
                            for (name, _) in files.iter() {
                                print!("{} ", name);
                            }
                        }
                        Err(_) => print!("`{}`: No such directory", fname),
                    },
                    path,
                )
            } else {
                print!("unspecified directory name");
            }
        }
        _ => print!("invalid arguments"),
    }
}

fn exec<'a>(mut _commands: impl Iterator<Item = &'a str>) {
    let f = crate::exec::compile_onlyhlt();
    f();
}
