use alloc::{ vec::Vec, boxed::Box,};
use super::structs::*;
use super::timer::{ Timer, Action};
use super::scheduler::Scheduler;
use super::Tid;

struct ThreadInfo {
   status : Status,
   present : bool,
   waiter : Option<Tid>,
   thread : Option<Box<Thread>>,
}

pub struct ThreadPool {
    threads: Vec<Option<ThreadInfo>>,    // 线程信号量的向量
    scheduler: Box<Scheduler>,      //　线程调度器
    timer: Box<Timer>,     // 时钟
}

impl ThreadPool{
    pub fn new(size : usize, scheduler : impl Scheduler + 'static) -> Self{
        ThreadPool {
            threads : {
                let mut th = Vec::new();
                th.resize_with(size, Default::default);
                th
            },
            scheduler : Box::new(scheduler),
            timer : Box::new(Timer::new()),
        }
    }

    fn alloc_tid(&self) -> Tid {
        for (i, info) in self.threads.iter().enumerate() {
            if info.is_none() || !info.as_ref().unwrap().present{
                return i;
            }
        }
        panic!("alloc tid failed !");
    }

    pub fn add(&mut self, _thread : Box<Thread>) {
        let tid = self.alloc_tid();
        self.threads[tid] = Some(ThreadInfo{
            status : Status::Ready,
            present : true,
            waiter : None,
            thread : Some(_thread),
        });
        self.scheduler.push(tid);
        println!("the tid to alloc : {}", tid);
    }

    pub fn tick(&mut self) -> bool{
        // 增加ｔｉｍｅｒ中的计时
        self.timer.tick();
        while let Some(action) = self.timer.pop() {
            match action {
                Action::Wakeup(tid) => {
                    self.wakeup(tid);
                },
            };
        }
        // 通知调度器时钟周期加一，询问是否需要调度
        self.scheduler.tick()
    }

    pub fn retrieve(&mut self, tid : Tid, thread : Box<Thread> ) {
        let mut proc = self.threads[tid].as_mut().expect("thread not exits !");
        if proc.present {
            match proc.status {
                Status::Sleeping => {
                    proc.thread = Some(thread);
                },
                Status::Ready => {
                    proc.thread = Some(thread);
                    proc.status = Status::Ready;
                    self.scheduler.push(tid);
                },
                _ => {},
            }
        }
        // set the state for stoped thread
    }

    pub fn acquire(&mut self) -> Option<(Tid, Box<Thread>)> {
        if let Some(tid) = self.scheduler.pop() {
            let mut proc = self.threads[tid].as_mut().expect("thread not exits !");
            proc.status = Status::Running(tid);
            return Some((tid, proc.thread.take().expect("thread does not exit ")));
        }else{
            return None;
        }
    }

    pub fn exit(&mut self, tid : Tid, code : usize) {
        self.threads[tid] = Some(ThreadInfo{
            status : Status::Ready,
            present : false,
            waiter : None,
            thread : None,
        });
        self.scheduler.exit(tid);
        println!("exit code : {}", code);
    }

    pub fn sleep(&mut self, tid : Tid, time : usize) {
        let proc = self.threads[tid].as_mut().expect("thread not exits");
        if proc.present {
            proc.status = Status::Sleeping;
            self.timer.push(Action::Wakeup(tid), time);
        }else{
            panic!("try to sleep an null thread !");
        }
    }

    fn wakeup(&mut self, tid : Tid) {
        //println!("into wakeup");
        let proc = self.threads[tid].as_mut().expect("thread not exits");
        if proc.present {
            proc.status = Status::Ready;
            self.scheduler.push(tid);
            //println!("wakeup {}", tid);
        }else{
            panic!("try to sleep an null thread !");
        }
    }
}
