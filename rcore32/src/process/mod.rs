mod interrupt;
mod thread_pool;
mod timer;
mod structs;
mod scheduler;

use self::structs::Thread;
use super::memory::{ paging::active_table, };
use self::thread_pool::ThreadPool;
use self::scheduler::RRScheduler;
use spin::Mutex;

static THREADPOOL : ThreadPool = ThreadPool::new();

pub fn init() {
    println!("+------ now to initialize process ------+");
    let scheduler = RRScheduler::new(1);
    unsafe{
        THREADPOOL.init(10, scheduler);
    }
    let mut thread0 = unsafe{ Thread::new_kernel(hello_thread, 0) };
    THREADPOOL.add(thread0);
    let mut thread1 = unsafe{ Thread::new_kernel(hello_thread, 1) };
    THREADPOOL.add(thread1);
    let mut thread2 = unsafe{ Thread::new_kernel(hello_thread, 2) };
    THREADPOOL.add(thread2);
    let mut thread3 = unsafe{ Thread::new_kernel(hello_thread, 3) };
    THREADPOOL.add(thread3);
    let mut thread4 = unsafe{ Thread::new_kernel(hello_thread, 4) };
    THREADPOOL.add(thread4);
    THREADPOOL.run();
}

use riscv::register::{scause::Scause, sstatus, sstatus::Sstatus};
use riscv::register::sie;
#[no_mangle]
pub extern "C" fn hello_thread(_arg : usize) -> ! {
    //for i in 0.._arg {
        //println!("hello thread");
    //}
    loop{
        println!("this is thread {}", _arg);
        //println!("hello thread");
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

    fn top(&self) -> usize {
        self.0 + STACK_SIZE
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        use alloc::alloc::{dealloc, Layout};
        unsafe {
            dealloc(
                self.0 as _,
                Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap(),
            );
        }
    }
}



pub type ExitCode = usize;
pub type Tid = usize;
pub type Pid = usize;

pub fn tick() {
    THREADPOOL.tick();
}
