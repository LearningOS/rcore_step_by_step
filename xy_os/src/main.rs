#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(global_asm)]

use core::panic::PanicInfo;
use bbl::sbi;

global_asm!(include_str!("arch/riscv32/boot/entry.asm"));

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}//

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    for &c in HELLO {
        sbi::console_putchar(c as usize);
    }
    loop {}
}

#[no_mangle]
pub extern fn abort() {
    panic!("abort!");
}
