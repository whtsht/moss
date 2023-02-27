#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(moss::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use moss::{hlt_loop, println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    moss::init();

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
