#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(moss::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use moss::{hlt_loop, keyboard::print_keypresses, println, task::Task};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    moss::init(boot_info).expect("failed to initialize kernel");

    #[cfg(test)]
    test_main();

    moss::task::add(Task::new(hello()));
    moss::task::add(Task::new(print_keypresses()));
    moss::task::run();
}

async fn hello() {
    println!("hello");
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    moss::test_panic_handler(info)
}
