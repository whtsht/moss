use alloc::alloc::{alloc, Layout};

pub fn compile_identity() -> fn(i32) -> i32 {
    unsafe {
        let mut compiler = Compiler::new();

        compiler.push(0x48);
        compiler.push(0x8b);
        compiler.push(0xc7);

        compiler.push(0xc3);

        core::mem::transmute(compiler.start_p())
    }
}

pub fn compile_onlyhlt() -> fn() {
    unsafe {
        let mut compiler = Compiler::new();
        compiler.push(0xf4);
        compiler.push(0xeb);
        compiler.push(0xfd);

        core::mem::transmute(compiler.start_p())
    }
}

pub struct Compiler {
    start_p: *mut u8,
    current_p: *mut u8,
}

impl Compiler {
    pub unsafe fn new() -> Self {
        let memory = alloc(Layout::from_size_align(4096, 4096).unwrap());

        Self {
            start_p: memory,
            current_p: memory,
        }
    }

    pub unsafe fn push(&mut self, code: u8) {
        core::ptr::write(self.current_p, code);
        self.current_p = (self.current_p as usize + 1) as *mut u8;
    }

    pub unsafe fn push_codes(&mut self, codes: &[u8]) {
        for code in codes.into_iter() {
            core::ptr::write(self.current_p, *code);
            self.current_p = (self.current_p as usize + 1) as *mut u8;
        }
    }

    pub fn start_p(&self) -> *mut u8 {
        self.start_p
    }

    pub fn current_p(&self) -> *mut u8 {
        self.current_p
    }
}
