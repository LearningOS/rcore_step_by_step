#![allow(dead_code)]

use crate::context::TrapFrame;
use riscv::register::{stvec, sscratch, sie};

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
