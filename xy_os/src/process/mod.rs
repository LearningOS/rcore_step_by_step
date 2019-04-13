mod structs;
mod scheduler;
mod processor;
mod thread_pool;

use structs::Thread;
use alloc::boxed::Box;
use processor::Processor;
use thread_pool::ThreadPool;
use self::scheduler::Scheduler;

pub type Tid = usize;
pub type ExitCode = usize;

static CPU: Processor = Processor::new();

pub fn tick() {
    CPU.tick();
}

pub fn init() {
    println!("+------ now to initialize process ------+");
    let scheduler = Scheduler::new(1);
    let thread_pool = ThreadPool::new(100, scheduler);
    CPU.init(Thread::new_idle(), Box::new(thread_pool));
    let thread0 = Thread::new_kernel(hello_thread, 0);
    CPU.add_thread(thread0);
    let thread1 = Thread::new_kernel(hello_thread, 1);
    CPU.add_thread(thread1);
    let thread2 = Thread::new_kernel(hello_thread, 2);
    CPU.add_thread(thread2);
    let thread3 = Thread::new_kernel(hello_thread, 3);
    CPU.add_thread(thread3);
    let thread4 = Thread::new_kernel(hello_thread, 4);
    CPU.add_thread(thread4);
    CPU.run();
}

#[no_mangle]
pub extern "C" fn hello_thread(arg: usize) -> ! {
    println!("hello thread");
    println!("arg is {}", arg);
    for i in 0..100 {
        println!("{}{}{}{}{}{}{}{}", arg, arg, arg, arg, arg, arg, arg, arg);
        for j in 0..10000 {
        }
    }
    println!("end of thread {}", arg);
    CPU.exit(0)
}
