use alloc::{ boxed::Box,};
use core::{ cell::UnsafeCell, };
use super::interrupt::*;
use super::structs::*;
use super::thread_pool::ThreadPool;
use super::Tid;

pub struct ProcessorInner{
    pool : Box<ThreadPool>,
    idle : Box<Thread>,
    current : Option<(Tid, Box<Thread>)>,
}

pub struct Processor{
    inner : UnsafeCell<Option<ProcessorInner>>,
}

unsafe impl Sync for Processor {}

impl Processor {
    pub const fn new() -> Self{
        Processor {
            inner : UnsafeCell::new(None),
        }
    }

    pub unsafe fn init(&self, idle : Box<Thread>, pool : Box<ThreadPool> ) {
        *self.inner.get() = Some(ProcessorInner{
            pool,
            idle,
            current : None,
        });
    }

    fn inner(&self) -> &mut ProcessorInner {
        unsafe { &mut *self.inner.get() }
            .as_mut()
            .expect("Processor is not initialized")
    }

    pub fn add_thread(&self, thread : Box<Thread>) {
        self.inner().pool.add(thread);
    }

    pub fn tick(&self) {
        let inner = self.inner();
        if inner.pool.tick() && !inner.current.is_none() {
            unsafe{
                let flags = disable_and_store();
                inner
                    .current
                    .as_mut()
                    .unwrap()
                    .1
                    .switch_to(&mut inner.idle);
                restore(flags);
            }
        }
    }

    pub fn run(&self) -> !{
        let inner = self.inner();
        unsafe{
            disable_and_store();
        }
        loop{
            if let Some(proc) = inner.pool.acquire() {
                inner.current = Some(proc);

                unsafe{ inner.idle.switch_to(&mut *inner.current.as_mut().unwrap().1);}

                let (tid, thread) = inner.current.take().unwrap();

                //println!("{} ran just now", tid);

                inner.pool.retrieve(tid, thread);
            }else{
                unsafe{
                    enable_and_wfi();
                }
                //println!("no thread to run");
                unsafe{
                    disable_and_store();
                }
            }
        }
    }

    pub fn exit(&self, code : usize) {
        let inner = self.inner();
        let tid = inner.current.as_ref().unwrap().0;
        inner.pool.exit(tid, code);
        self.yield_now();
    }

    pub fn sleep(&self, time : usize) {
        let inner = self.inner();
        let tid = inner.current.as_ref().unwrap().0;
        inner.pool.sleep(tid, time);
        self.yield_now();
    }

    pub fn yield_now(&self) {
        let inner = self.inner();
        unsafe {
            let flags = disable_and_store(); // 禁止中断，获取当前ｓｓｔａｔｕｓ的状态并保存。
            inner
                .current
                .as_mut()
                .unwrap()
                .1
                .switch_to(&mut *inner.idle);   // 转到ｉｄｌｅ线程执行
            restore(flags);  // 使能中断，恢复ｓｓｔａｔｕｓ的状态
        }
    }
}
