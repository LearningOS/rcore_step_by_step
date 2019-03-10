#![feature(asm)]
#![feature(global_asm)]
#![no_std]
#![no_main]
#![allow(dead_code)]

use core::panic::PanicInfo;

global_asm!(include_str!("boot/entry.asm"));


#[no_mangle]
pub extern "C" fn boot_main() -> ! {
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
