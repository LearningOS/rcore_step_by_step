mod interrupt;
mod thread_pool;
mod timer;
mod structs;
mod scheduler;
mod processor;

use self::structs::Thread;
use self::thread_pool::ThreadPool;
use self::scheduler::RRScheduler;
use self::processor::Processor;
use alloc::boxed::Box;

static CPU : Processor = Processor::new();

pub fn init() {
    println!("+------ now to initialize process ------+");
    let scheduler = RRScheduler::new(1);
    let thread_pool = ThreadPool::new(100, scheduler);
    unsafe{
        CPU.init(Thread::new_init(), Box::new(thread_pool));
    }
    let thread0 = unsafe{ Thread::new_kernel(hello_thread, 0) };
    CPU.add_thread(thread0);
    let thread1 = unsafe{ Thread::new_kernel(hello_thread, 1) };
    CPU.add_thread(thread1);
    let thread2 = unsafe{ Thread::new_kernel(hello_thread, 2) };
    CPU.add_thread(thread2);
    let thread3 = unsafe{ Thread::new_kernel(hello_thread, 3) };
    CPU.add_thread(thread3);
    let thread4 = unsafe{ Thread::new_kernel(hello_thread, 4) };
    CPU.add_thread(thread4);
    CPU.run();
}

#[no_mangle]
pub extern "C" fn hello_thread(_arg : usize) -> ! {
    if _arg == 1 {
        CPU.sleep(100);
    }
    loop{
        //println!("this is thread {}", _arg);
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
    CPU.tick();
}

//pub(crate) fn current_tid() -> Tid {
    //THREADPOOL.current_tid()
//}

//pub fn sleep(time : usize) {
    //THREADPOOL.sleep(current_tid(), time);
//}
