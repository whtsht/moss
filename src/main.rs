#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(naked_functions)]
#![test_runner(moss::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use moss::{hlt_loop, keyboard::print_keypresses, print, println};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    moss::init(boot_info).expect("failed to initialize kernel");

    #[cfg(test)]
    test_main();

    moss::task::add(moss::task::Task::new(dot()));
    moss::task::add(moss::task::Task::new(world()));
    moss::task::add(moss::task::Task::new(print_keypresses()));

    moss::task::run();
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

async fn dot() {
    use x86_64::instructions::interrupts;
    let mut tmp = 0;
    loop {
        let a = interrupts::without_interrupts(|| *moss::interrupts::GLOBAL_COUNTER.lock());
        if tmp + 10 <= a {
            print!(".");
            tmp = a;
        } else {
            (async {}).await;
        }
    }
}

async fn world() {
    println!("world");
}
