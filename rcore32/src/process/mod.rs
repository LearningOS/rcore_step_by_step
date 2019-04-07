mod interrupt;
mod thread_pool;
mod timer;
mod structs;
mod scheduler;

use self::structs::Thread;
use super::memory::{ paging::active_table, };

pub fn init() {
    println!("+------ now to initialize process ------+");
    unsafe{
        let mut loop_thread = Thread::new_init();
        let mut hello_thread = Thread::new_kernel(hello_thread, 5);
        loop_thread.switch_to(&mut hello_thread);
    }
}

pub fn tick() {
    println!("a tick !");
}

#[no_mangle]
pub extern "C" fn hello_thread(_arg : usize) -> ! {
    for i in 0.._arg {
        println!("hello thread");
    }
    loop{
    }
}

pub struct KernelStack(usize);
const STACK_SIZE : usize = 0x8000;

impl KernelStack {
    pub fn new() -> Self{
        use alloc::alloc::{alloc, Layout};
        let bottom =
            unsafe { alloc(Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap()) } as usize;
        KernelStack(bottom)
    }

    fn top(self) -> usize {
        self.0 + STACK_SIZE
    }
}


pub type ExitCode = usize;
pub type Tid = usize;
pub type Pid = usize;

