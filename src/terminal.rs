use crate::println;

use alloc::string::String;
use pc_keyboard::DecodedKey;
use spin::Mutex;

use crate::print;

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
        print!(">");
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
            "echo" => {
                for args in commands {
                    for c in args.chars() {
                        print!("{}", c);
                    }
                }
                print!("\n");
            }
            _ => println!("command not found: {}", command),
        }
    }
}
