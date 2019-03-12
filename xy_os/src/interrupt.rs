use crate::context::TrapFrame;
use riscv::register::stvec;

#[no_mangle]
pub fn init() {
    extern {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
    }
}

#[no_mangle]
pub extern "C" fn rust_trap(tf: &mut TrapFrame) {
    println!("here a trap");
    tf.increase_sepc();
}
