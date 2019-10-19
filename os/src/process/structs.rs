extern crate alloc;
use crate::context::Context;
use alloc::alloc::{alloc, dealloc, Layout};
use riscv::register::satp;

pub struct Thread {
    pub context: Context, // 线程相关的上下文
    pub kstack: KernelStack, // 线程对应的内核栈
}

impl Thread {
    pub fn new_idle() -> Thread {
        unsafe {
            Thread {
                context: Context::null(),
                kstack: KernelStack::new(),
            }
        }
    }

    pub fn new_kernel(entry: extern "C" fn(usize) -> !, arg: usize) -> Thread {
        unsafe {
            let kstack_ = KernelStack::new();
            Thread {
                context: Context::new_kernel_thread(entry, arg, kstack_.top(), satp::read().bits()),
                kstack: kstack_,
            }
        }
    }

    pub fn switch_to(&mut self, target: &mut Thread) {
       unsafe {
           self.context.switch(&mut target.context);
       }
   }
}

pub struct KernelStack(usize);
const STACK_SIZE: usize = 0x8000;

impl KernelStack {
    pub fn new() -> KernelStack {
        let bottom =
            unsafe {
                alloc(Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap())
            } as usize;
        KernelStack(bottom)
    }

    pub fn top(&self) -> usize {
        self.0 + STACK_SIZE
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.0 as _,
                Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap()
            );
        }
    }
}

