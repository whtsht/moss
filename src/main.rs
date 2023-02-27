#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(moss::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use moss::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    moss::init();

    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    println!("It did not crash");

    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    moss::test_panic_handler(info)
}
