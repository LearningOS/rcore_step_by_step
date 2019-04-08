use alloc::{ vec::Vec, boxed::Box};
use spin::{Mutex, MutexGuard};
use super::structs::*;
use super::timer::Timer;
use super::scheduler::{ Scheduler, RRScheduler};
use super::Tid;

struct ThreadInfo {
   status : Status,
   next_status : Status,
   waiter : Option<Tid>,
   context : Arc<Mutex<Context>>,
}

pub struct ThreadPool {
    threads: Vec<Mutex<Option<ThreadInfo>>>,    // 线程信号量的向量
    scheduler: Box<Scheduler>,      //　线程调度器
    timer: Mutex<Timer>,     // 时钟
}

impl ThreadPool{
    pub fn new(size : usize, scheduler : impl Scheduler + 'static) -> Self{
        ThreadPool {
            threads : {
                let mut th = Vec::new();
                th.resize_with(size, Default::default);
                th
            },
            scheduler: Box::new(scheduler),
            timer : Mutex::new(Timer::new()),
        }
    }

    fn alloc_tid(&self) -> (Tid, MutexGuard<Option<ThreadInfo>>) {
        for (i, proc) in self.threads.iter().enumerate() {
            let thread = proc.lock();
            if thread.is_none() {
                return (i, thread);
            }
        }
        panic!("fault !");
    }

    fn add(&self, _thread : mut Thread) {
        let (tid, mut thread) = self.alloc_tid();
        _thread.set_tid(tid);
        *thread = Some(ThreadInfo{
            status : Status::Ready,
            next_status : Status::Ready,
            waiter : None,
            context : Arc::new(Mutex::new(_thread.context)),
        });
    }

    fn tick(&self) -> bool {
        // 增加ｔｉｍｅｒ中的计时
        // 通知调度器时钟周期加一，询问是否需要调度
    }
}
