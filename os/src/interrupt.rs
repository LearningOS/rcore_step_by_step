use riscv::register::{ stvec, sscratch, sstatus };
use riscv::register::scause::{ Trap, Exception, Interrupt };
use crate::clock::{ TICK, clock_set_next_event };
use crate::context::TrapFrame;

global_asm!(include_str!("trap/trap.asm"));

pub fn init() {
    extern {
        fn __alltraps();
    }
    unsafe {
        sscratch::write(0); // 给中断 asm 初始化
        sstatus::set_sie();
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
    }
    println!("++++setup interrupt !++++");
}

#[no_mangle]
pub fn rust_trap(tf: &mut TrapFrame) {
    match tf.scause.cause() {
        Trap::Exception(Exception::Breakpoint) => breakpoint(),
        Trap::Interrupt(Interrupt::SupervisorTimer) => super_timer(),
        Trap::Exception(Exception::InstructionPageFault) => page_fault(tf),
        Trap::Exception(Exception::LoadPageFault) => page_fault(tf),
        Trap::Exception(Exception::StorePageFault) => page_fault(tf),
        _ => panic!("unexpected trap"),
    }
}

fn page_fault(tf: &mut TrapFrame) {
    println!("{:?} @ {:#x}", tf.scause.cause(), tf.stval);
    panic!("page fault");
}

fn breakpoint() {
    panic!("a breakpoint set by kernel");
}

fn super_timer() {
    // 响应当前时钟中断的同时，手动设置下一个时钟中断
    clock_set_next_event();
    unsafe{
        TICK = TICK + 1;
        if TICK % 100 == 0 {
            println!("100 ticks!");
        }
    }
}
