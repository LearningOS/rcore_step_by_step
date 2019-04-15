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

extern "C" {
    fn _user_img_start();
    fn _user_img_end();
}

pub fn init() {
    println!("+------ now to initialize process ------+");
    let scheduler = Scheduler::new(50);
    let thread_pool = ThreadPool::new(100, scheduler);
    CPU.init(Thread::new_idle(), Box::new(thread_pool));
    let data = unsafe{
        ::core::slice::from_raw_parts(
            _user_img_start as *const u8,
            _user_img_end as usize - _user_img_start as usize,
        )
    };
    let user = unsafe{ Thread::new_user(data) };
    CPU.add_thread(user);
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
