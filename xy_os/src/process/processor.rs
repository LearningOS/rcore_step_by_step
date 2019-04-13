use core::cell::UnsafeCell;
use alloc::boxed::Box;
use crate::process::Tid;
use crate::process::structs::*;
use crate::process::thread_pool::ThreadPool;
use crate::interrupt::{ disable_and_store, enable_and_wfi, restore };

pub struct ProcessorInner {
    pool: Box<ThreadPool>,
    idle: Box<Thread>,
    current: Option<(Tid, Box<Thread>)>,
}

pub struct Processor {
    inner: UnsafeCell<Option<ProcessorInner>>,
}

unsafe impl Sync for Processor {}

impl Processor {
    pub const fn new() -> Processor {
        Processor {
            inner: UnsafeCell::new(None),
        }
    }

    pub fn init(&self, idle: Box<Thread>, pool: Box<ThreadPool> ) {
        unsafe {
            *self.inner.get() = Some(ProcessorInner{
                pool,
                idle,
                current: None,
            });
        }
    }

    fn inner(&self) -> &mut ProcessorInner {
        unsafe { &mut *self.inner.get() }
            .as_mut()
            .expect("Processor is not initialized")
    }

    pub fn add_thread(&self, thread: Box<Thread>) {
        self.inner().pool.add(thread);
    }

    pub fn run(&self) -> !{
        let inner = self.inner();
        // 关闭中断，防止此时产生中断异常导致线程切换出错。
        disable_and_store();
        // 循环从线程池中寻找可调度线程
        loop {
            // 如果存在需要被调度的线程
            if let Some(proc) = inner.pool.acquire() {
                inner.current = Some(proc);
                // 切换至需要被调度的线程
                inner.idle.switch_to(&mut *inner.current.as_mut().unwrap().1);
                // 上一个线程已经结束或时间片用完，切换回 idle 线程
                let (tid, thread) = inner.current.take().unwrap();
                println!("thread {} ran just now", tid);
                // 将上一个线程放回线程池中
                inner.pool.retrieve(tid, thread);
            } else {
                // 开启中断并等待中断产生
                enable_and_wfi();
                // 关闭中断，从线程池中寻找可调度线程
                disable_and_store();
            }
        }
    }

    pub fn tick(&self) {
        let inner = self.inner();
        if !inner.current.is_none() {
            if inner.pool.tick() {
                let flags = disable_and_store();
                inner
                    .current
                    .as_mut()
                    .unwrap()
                    .1
                    .switch_to(&mut inner.idle);
                // 恢复原先的中断状态
                restore(flags);
            }
        }
    }
    
    pub fn exit(&self, code: usize) -> ! {
        let inner = self.inner();
        let tid = inner.current.as_ref().unwrap().0;
        inner.pool.exit(tid, code);
        inner
            .current
            .as_mut()
            .unwrap()
            .1
            .switch_to(&mut inner.idle);
        loop {}
    }
}
