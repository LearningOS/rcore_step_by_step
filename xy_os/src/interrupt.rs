use crate::context::TrapFrame;
use riscv::register::{stvec, sstatus};

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
    extern {
        fn __alltraps();
    }
    unsafe {
        sstatus::set_sie();
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
    }
    println!("++++setup interrupt !++++");
}

use riscv::register::mcause::Trap;
use riscv::register::mcause::Exception;
use riscv::register::mcause::Interrupt;
use crate::clock::{TICK, clock_set_next_event};

#[no_mangle]
pub extern "C" fn rust_trap(tf: &mut TrapFrame) {
    match tf.scause.cause() {
        Trap::Exception(Exception::Breakpoint) => breakpoint(),
        Trap::Interrupt(Interrupt::SupervisorTimer) => super_timer(),
        _ => panic!("..."),
    }
    // tf.increase_sepc();
}

fn super_timer() {
    //响应当前时钟中断的同时，手动设置下一个时钟中断。这一函数调用同时清除了MTIP，使得CPU知道当前这个中断被正确处理。
    clock_set_next_event(); 
    unsafe{
        TICK = TICK + 1;
        if(TICK % 100 == 0) {
            println!("ticks 100!");
        }
    }
}

fn breakpoint() {
    panic!("a breakpoint set by kernel");
}