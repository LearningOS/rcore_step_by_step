use crate::context::TrapFrame;
use crate::memory::{do_pgfault, PageFault};
use riscv::register::{stvec, sstatus};

global_asm!(include_str!("trap/trap.asm"));

#[no_mangle]
pub fn init() {
    extern {
        fn __alltraps();
    }
    unsafe {
        sstatus::set_sie();
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
    }
    println!("++++setup interrupt !++++");
}

use riscv::register::scause::Trap;
use riscv::register::scause::Exception;
use riscv::register::scause::Interrupt;
use crate::clock::{TICK, clock_set_next_event};

#[no_mangle]
pub extern "C" fn rust_trap(tf: &mut TrapFrame) {
    match tf.scause.cause() {
        Trap::Exception(Exception::Breakpoint) => breakpoint(),
        Trap::Interrupt(Interrupt::SupervisorTimer) => super_timer(),
        Trap::Exception(Exception::LoadPageFault) => do_pgfault(tf, PageFault::LoadPageFault),
        Trap::Exception(Exception::StorePageFault) => do_pgfault(tf, PageFault::StorePageFault),
        _ => panic!("..."),
    }
    // tf.increase_sepc();
}

use crate::process::tick;
fn super_timer() {
    // 响应当前时钟中断的同时，手动设置下一个时钟中断。这一函数调用同时清除了MTIP，使得CPU知道当前这个中断被正确处理。
    clock_set_next_event(); 
    unsafe{
        TICK = TICK + 1;
        if TICK % 100 == 0 {
            println!("ticks 100!");
        }
    }
    // tick();
}

fn breakpoint() {
    panic!("a breakpoint set by kernel");
}


#[inline(always)]
pub fn enable_and_wfi() {    // 使能中断并等待中断
    unsafe {
        asm!("csrsi sstatus, 1 << 1; wfi" :::: "volatile");
    }
}

#[inline(always)]
pub fn disable_and_store() -> usize {    // 禁用中断并返回当前中断状态
    let sstatus: usize;
    unsafe {
        asm!("csrci sstatus, 1 << 1" : "=r"(sstatus) ::: "volatile");
    }
    sstatus & (1 << 1)
}

#[inline(always)]
pub fn restore(flags: usize) {    // 根据 flag 设置中断
    unsafe {
        asm!("csrs sstatus, $0" :: "r"(flags) :: "volatile");
    }
}
