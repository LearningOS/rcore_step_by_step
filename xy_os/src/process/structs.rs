use crate::context::Context;

pub struct Thread {
    context: Context, // 线程相关的上下文
    kstack: KernelStack, // 线程对应的内核栈
}

use riscv::register::satp;
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
            let _kstack = KernelStack::new();
            Thread {
                context: Context::new_kernel_thread(entry, arg, _kstack.top(), satp::read().bits()),
                kstack: _kstack,
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

use alloc::alloc::{alloc, dealloc, Layout};
impl KernelStack {

    pub fn new() -> KernelStack {
        let bottom =
            unsafe {
                alloc(Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap())
            } as usize;
        KernelStack(bottom)
    }

    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.0 as _,
                Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap()
            );
        }
    }

    pub fn top(&self) -> usize {
        self.0 + STACK_SIZE
    }

}