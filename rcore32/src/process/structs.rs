use alloc::{ vec::Vec, sync::Arc};
use crate::context::Context;
use super::{KernelStack, Tid, Pid, ExitCode};
use crate::memory::current_root;

use spin::Mutex;

pub enum Status {
    Ready,
    Running(Tid),
    Sleeping,
    Exited(ExitCode),
}


pub struct Thread {
    pub status : Status,    // 线程状态，包括就绪、运行、休眠、结束等
    pub context : Context,  // 线程相关的上下文
    pub waiter : Option<Tid>, // 父线程的ｔｉｄ
    pub proc : Option<Arc<Mutex<Process>>>,  // 线程对应的进程
}

impl Thread {
    pub unsafe fn new_init() -> Self {
        Thread {
            status : Status::Ready,
            context : Context::null(),
            waiter : None,
            proc : None,
        }
    }

    pub unsafe fn new_kernel(entry : extern "C" fn(usize) -> !, arg : usize) -> Self {
        let kstack = KernelStack::new();
        Thread {
            status : Status::Ready,
            context : Context::new_kernel_thread(entry, arg, kstack.top(), current_root()),
            waiter : None,
            proc : None,
        }
    }

    pub unsafe fn switch_to(&mut self, target : &mut Thread) {
        self.context.switch(&mut target.context);
    }
}

pub struct Process {
    //vm : MemorySet,

    //pub pid : Pid,
    //pub parent : Option<Arc<Mutex<Process>>>,
    //pub children : Vec<Weak<Mutex<Process>>>,
    //pub threads : Vec<Tid>,
}
