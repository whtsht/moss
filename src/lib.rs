#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::BootInfo;
use core::panic::PanicInfo;
use x86_64::VirtAddr;

pub mod error;
pub mod gdt;
pub mod interrupts;
pub mod keyboard;
pub mod paging;
pub mod serial;
pub mod vga_buffer;

use error::Result;

pub fn init(boot_info: &'static BootInfo) -> Result<()> {
    gdt::init();
    interrupts::init_idt();
    unsafe { keyboard::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);

    unsafe { paging::init(physical_memory_offset) };

    let mut mapper = paging::get_mapper()?;
    let mut allocator = unsafe { paging::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    paging::make_identity_mapping(&mut mapper, &mut allocator, 0xfee00000, 1).unwrap();

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
    // like before
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
