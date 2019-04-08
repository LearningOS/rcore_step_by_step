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
    pub context : Context,  // 线程相关的上下文
    pub kstack : KernelStack,   // 线程对应的内核栈
    pub proc : Option<Arc<Mutex<Process>>>,  // 线程对应的进程
}

impl Thread {
    pub unsafe fn new_init() -> Self {
        Thread {
            context : Context::null(),
            kstack : KernelStack::new(),
            proc : None,
        }
    }

    pub unsafe fn new_kernel(entry : extern "C" fn(usize) -> !, arg : usize) -> Self {
        let kstack = KernelStack::new();
        Thread {
            context : Context::new_kernel_thread(entry, arg, kstack.top(), current_root()),
            kstack : kstack,
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
