#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(custom_test_frameworks)]
#![feature(naked_functions)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::BootInfo;
use core::panic::PanicInfo;
use x86_64::VirtAddr;

pub mod allocator;
pub mod error;
pub mod gdt;
pub mod interrupts;
pub mod keyboard;
pub mod paging;
pub mod serial;
pub mod task;
pub mod terminal;
pub mod vga_buffer;

use error::Result;

pub fn init(boot_info: &'static BootInfo) -> Result<()> {
    gdt::init();

    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);

    unsafe { paging::init(physical_memory_offset) };

    let mut mapper = paging::get_mapper()?;
    let mut allocator = unsafe { paging::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    paging::make_identity_mapping(&mut mapper, &mut allocator, 0xfee00000, 1).unwrap();
    allocator::init_heap(&mut *mapper, &mut allocator)?;

    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop()
}

#[cfg(test)]
use bootloader::entry_point;

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo test`
#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

use alloc::{boxed::Box, vec, vec::Vec};
use core::arch::asm;
use core::mem;
use spin::Once;
use x86_64::registers::control::Cr3;
pub static TASK_MAIN: Once<ContextTask> = Once::new();
pub static TASK_B: Once<ContextTask> = Once::new();

#[derive(Debug)]
#[repr(C, align(16))]
pub struct TaskContext {
    // offset : 0x00
    cr3: u64,
    rip: u64,
    rflags: u64,
    reserved1: u64,
    // offset : 0x20
    cs: u64,
    ss: u64,
    fs: u64,
    gs: u64,
    // offset : 0x40
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rdi: u64,
    rsi: u64,
    rsp: u64,
    rbp: u64,
    // offset : 0x80
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    // offset : 0xc0
    fxsave_area: [u8; 512],
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C, align(16))]
pub struct TaskStackElement {
    _dummy: [u64; 2],
}

pub struct ContextTask {
    ctx: Box<TaskContext>,
    stack: Vec<TaskStackElement>,
}

pub type EntryPoint = extern "C" fn(arg0: u64, arg1: u64);

impl ContextTask {
    pub fn new(entry_point: EntryPoint, arg0: u64, arg1: u64) -> Self {
        let stack_size = 1024 * 8;
        let stack_elem_size = mem::size_of::<TaskStackElement>();
        let mut task = Self {
            ctx: Box::new(unsafe { mem::zeroed() }),
            stack: vec![
                TaskStackElement::default();
                (stack_size + stack_elem_size - 1) / stack_elem_size
            ],
        };

        let selectors = crate::gdt::selectors();

        task.ctx.rip = entry_point as *const u8 as u64;
        task.ctx.rdi = arg0;
        task.ctx.rsi = arg1;

        task.ctx.cr3 = Cr3::read().0.start_address().as_u64();
        task.ctx.rflags = 0x202;
        task.ctx.cs = u64::from(selectors.code_selector.0);
        task.ctx.ss = u64::from(selectors.stack_selector.0);
        task.ctx.rsp = unsafe { (task.stack.as_ptr() as *const u8).add(stack_size - 8) as u64 };
        assert!(task.ctx.rsp & 0xf == 8);

        task.ctx.fxsave_area[24..][..4].copy_from_slice(&0x1f80u32.to_le_bytes());

        task
    }

    pub fn switch(next: &'static ContextTask, current: &'static ContextTask) {
        switch_task(&next.ctx, &current.ctx);
    }
}

#[naked]
extern "C" fn switch_task(_next: &'static TaskContext, _current: &'static TaskContext) {
    unsafe {
        asm!(
            "mov [rsi + 0x40], rax",
            "mov [rsi + 0x48], rbx",
            "mov [rsi + 0x50], rcx",
            "mov [rsi + 0x58], rdx",
            "mov [rsi + 0x60], rdi",
            "mov [rsi + 0x68], rsi",
            //
            "lea rax, [rsp + 8]",
            "mov [rsi + 0x70], rax", // RIP
            "mov [rsi + 0x78], rbp",
            //
            "mov [rsi + 0x80], r8",
            "mov [rsi + 0x88], r9",
            "mov [rsi + 0x90], r10",
            "mov [rsi + 0x98], r11",
            "mov [rsi + 0xa0], r12",
            "mov [rsi + 0xa8], r13",
            "mov [rsi + 0xb0], r14",
            "mov [rsi + 0xb8], r15",
            //
            "mov rax, cr3",
            "mov [rsi + 0x00], rax", // CR3
            "mov rax, [rsp]",
            "mov [rsi + 0x08], rax", // RIP
            "pushfq",
            "pop QWORD PTR [rsi + 0x10]", // RFLAGS
            //
            "mov ax, cs",
            "mov [rsi + 0x20], rax",
            "mov bx, ss",
            "mov [rsi + 0x28], rbx",
            "mov cx, fs",
            "mov [rsi + 0x30], rcx",
            "mov dx, gs",
            "mov [rsi + 0x38], rdx",
            //
            "fxsave [rsi + 0xc0]",
            //
            // stack frame for iret
            "push QWORD PTR [rdi + 0x28]", // SS
            "push QWORD PTR [rdi + 0x70]", // RSP
            "push QWORD PTR [rdi + 0x10]", // RFLAGS
            "push QWORD PTR [rdi + 0x20]", // CS
            "push QWORD PTR [rdi + 0x08]", // RIP
            //
            // restore context
            "fxrstor [rdi + 0xc0]",
            //
            "mov rax, [rdi + 0x00]",
            "mov cr3, rax",
            "mov rax, [rdi + 0x30]",
            "mov fs, ax",
            "mov rax, [rdi + 0x38]",
            "mov gs, ax",
            //
            "mov rax, [rdi + 0x40]",
            "mov rbx, [rdi + 0x48]",
            "mov rcx, [rdi + 0x50]",
            "mov rdx, [rdi + 0x58]",
            "mov rsi, [rdi + 0x68]",
            "mov rbp, [rdi + 0x78]",
            "mov r8,  [rdi + 0x80]",
            "mov r9,  [rdi + 0x88]",
            "mov r10, [rdi + 0x90]",
            "mov r11, [rdi + 0x98]",
            "mov r12, [rdi + 0xa0]",
            "mov r13, [rdi + 0xa8]",
            "mov r14, [rdi + 0xb0]",
            "mov r15, [rdi + 0xb8]",
            //
            "mov rdi, [rdi + 0x60]",
            //
            "iretq",
            options(noreturn)
        );
    }
}

extern "C" fn _dummy_task(_arg0: u64, _arg1: u64) {
    panic!("dummy task called;")
}

extern "C" fn _task_b(_arg0: u64, _arg1: u64) {
    for i in 0..5 {
        println!("Hello from taskB {}", i);
        ContextTask::switch(TASK_MAIN.get().unwrap(), TASK_B.get().unwrap());
    }
}
