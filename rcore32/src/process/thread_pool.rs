use alloc::{ vec::Vec, boxed::Box, sync::Arc};
use core::{ cell::UnsafeCell, };
use spin::{Mutex, MutexGuard};
use super::structs::*;
use super::timer::Timer;
use super::scheduler::{ Scheduler, RRScheduler};
use super::Tid;
use super::interrupt::*;

struct ThreadInfo {
   status : Status,
   next_status : Status,
   waiter : Option<Tid>,
   thread : Option<Box<Thread>>,
}

struct ThreadPoolInner {
    threads: Vec<Mutex<Option<ThreadInfo>>>,    // 线程信号量的向量
    scheduler: Box<Scheduler>,      //　线程调度器
    timer: Mutex<Timer>,     // 时钟
    idle : Box<Thread>,
    current : Option<(Tid, Box<Thread>)>,
}

unsafe impl Sync for ThreadPool {}

pub struct ThreadPool {
    inner : UnsafeCell<Option<ThreadPoolInner>>,
}

impl ThreadPool{
    pub const fn new() -> Self {
        ThreadPool {
            inner : UnsafeCell::new(None),
        }
    }

    pub unsafe fn init(&self, size : usize, scheduler : impl Scheduler + 'static){
        *self.inner.get() = Some(ThreadPoolInner {
            threads : {
                let mut th = Vec::new();
                th.resize_with(size, Default::default);
                th
            },
            scheduler: Box::new(scheduler),
            timer : Mutex::new(Timer::new()),
            idle : Thread::new_init(),
            current : None,
        })
    }

    fn inner(&self) -> &mut ThreadPoolInner {
        unsafe { &mut *self.inner.get() }
            .as_mut()
            .expect("ThreadPool is not initialized")
    }

    fn alloc_tid(&self) -> (Tid, MutexGuard<Option<ThreadInfo>>) {
        let inner = self.inner();
        for (i, proc) in inner.threads.iter().enumerate() {
            let thread = proc.lock();
            if thread.is_none() {
                return (i, thread);
            }
        }
        panic!("fault !");
    }

    pub fn add(&self, _thread : Box<Thread>) {
        let (tid, mut thread) = self.alloc_tid();
        *thread = Some(ThreadInfo{
            status : Status::Ready,
            next_status : Status::Ready,
            waiter : None,
            thread : Some(_thread),
        });
        self.inner().scheduler.push(tid);
        println!("the tid to alloc : {}", tid);
    }

    pub fn tick(&self) {
        // 增加ｔｉｍｅｒ中的计时
        // 通知调度器时钟周期加一，询问是否需要调度
        println!("a tick in thread pool !");
        let mut inner = self.inner();
        if inner.scheduler.tick() {
            println!("here i guess");
            unsafe{
                inner
                    .current
                    .as_mut()
                    .unwrap()
                    .1
                    .switch_to(&mut inner.idle);
            }
        }
    }

    pub fn run(&self) -> !{
        let inner = self.inner();
        unsafe{
            disable_and_store();
        }
        loop{
            if let Some(tid) = inner.scheduler.pop() {
                println!("{} : next tid to run", tid);
                let mut info_lock = inner.threads[tid].lock();
                if let info = info_lock.as_mut().unwrap() {
                    if let Some(mut thread) = info.thread.take() {
                        inner.current = Some((tid, thread));
                        unsafe{ inner.idle.switch_to(&mut *inner.current.as_mut().unwrap().1);}
                        inner.scheduler.push(tid);
                    }
                }
            }else{
                unsafe{
                    enable_and_wfi();
                }
                unsafe{
                    disable_and_store();
                }
            }
        }
    }
}
