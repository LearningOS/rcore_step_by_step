#![allow(dead_code)]
use crate::context::TrapFrame;
use riscv::register::{stvec, sscratch, sie};

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

#[no_mangle]
pub fn init() {
    println!("start interrupt init !");
    extern {
        fn __alltraps();
    }
    unsafe {
        sscratch::write(0);
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
        sie::set_ssoft();
        sie::set_sext();
    }
    println!("finish interrupt init !");
}

#[no_mangle]
pub extern "C" fn rust_trap(tf: &mut TrapFrame) {
    println!("here a trap ! ");
    tf.increase_sepc();
}
