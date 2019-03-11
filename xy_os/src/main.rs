#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(global_asm)]

#[macro_use]
pub mod io;

use core::panic::PanicInfo;

global_asm!(include_str!("arch/riscv32/boot/entry.asm"));

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    let a = "Hello";
    let b = "World";
    println!("{}, {}!", a, b);
    panic!("End of rust_main");
}

#[no_mangle]
pub extern fn abort() {
    panic!("abort!");
}