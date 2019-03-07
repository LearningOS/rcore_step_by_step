#![feature(asm)]
#![feature(global_asm)]
#![no_std]
#![no_main]
mod io;
mod sbi;
use core::panic::PanicInfo;
use sbi::console_putchar as cprint;

global_asm!(include_str!("boot/entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("Hello World");
    //interrupt::init();
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern fn abort() {
    panic!("abort");
}
