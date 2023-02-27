#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(moss::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::vec;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use moss::{hlt_loop, println};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    moss::init(boot_info).expect("failed to initialize kernel");

    let v = vec![1, 2, 3];
    println!("Hello World {}", v[2]);

    #[cfg(test)]
    test_main();

    hlt_loop();
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
