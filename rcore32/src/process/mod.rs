mod interrupt;
mod thread_pool;
mod timer;
mod structs;

use super::context::Context;
use super::memory::paging::active_table;

pub fn init() {
    println!("+------ now to initialize process ------+");
    use alloc::alloc::{alloc, Layout};
    use riscv::register::satp;
    let bottom =
        unsafe { alloc(Layout::from_size_align(0x8000, 0x8000).unwrap()) } as usize;
    println!("bottom is {:#x} ", bottom);
    unsafe{
        let mut loop_context = Context::null();
        let mut hello_context = Context::new_kernel_thread(hello_thread, 0, bottom + 0x8000, satp::read().bits());
        loop_context.switch(&mut hello_context);
    }
}

pub fn tick() {
    println!("a tick !");
}

#[no_mangle]
pub extern "C" fn hello_thread(_arg : usize) -> ! {
    loop{
        println!("hello thread");
    }
}
