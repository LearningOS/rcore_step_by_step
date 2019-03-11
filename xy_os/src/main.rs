#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(global_asm)]
#![feature(asm)]

#[macro_use]
pub mod io;

pub mod context;
mod interrupt;

use core::panic::PanicInfo;

global_asm!(include_str!("boot/entry.asm"));

#[cfg(feature = "m_mode")]
global_asm!("
    .equ xstatus,   0x300
    .equ xscratch,  0x340
    .equ xepc,      0x341
    .equ xcause,    0x342
    .equ xtval,     0x343
    .macro XRET\n mret\n .endm
    .macro TEST_BACK_TO_KERNEL  // s0 == back to kernel?
        li   s3, 3 << 11
        and  s0, s1, s3         // mstatus.MPP = 3
    .endm
");
#[cfg(not(feature = "m_mode"))]
global_asm!("
    .equ xstatus,   0x100
    .equ xscratch,  0x140
    .equ xepc,      0x141
    .equ xcause,    0x142
    .equ xtval,     0x143
    .macro XRET\n sret\n .endm
    .macro TEST_BACK_TO_KERNEL
        andi s0, s1, 1 << 8     // sstatus.SPP = 1
    .endm
");
global_asm!(r"
    .equ XLENB,     4
    .equ XLENb,     32
    .macro LOAD a1, a2
        lw \a1, \a2*XLENB(sp)
    .endm
    .macro STORE a1, a2
        sw \a1, \a2*XLENB(sp)
    .endm
");
global_asm!(include_str!("trap/trap.asm"));

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
    interrupt::init();
    unsafe{
        asm!("ebreak\n"::::);
    }
    panic!("End of rust_main");
}

#[no_mangle]
pub extern fn abort() {
    panic!("abort!");
}