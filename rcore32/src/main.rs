#![feature(asm)]
#![feature(global_asm)]
#![no_std]
#![no_main]
mod sbi;
use core::panic::PanicInfo;
use sbi::console_putchar as con_put;

global_asm!(include_str!("boot/entry.asm"));

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    for (i, &byte) in HELLO.iter().enumerate() {
        con_put(byte as usize);
    }
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
