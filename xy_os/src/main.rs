#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(global_asm)]

#[macro_use]
extern crate libr;

use core::panic::PanicInfo;

global_asm!(include_str!("arch/riscv32/boot/entry.asm"));


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    let a = 111;
    let b = "666";
    libr::io::puts("000");
    print!("{}2{}", a, b);
    loop {}
}

#[no_mangle]
pub extern fn abort() {
    panic!("abort!");
}
