use super::Tid;
use alloc::collections::VecDeque;

pub trait Scheduler {
    fn push(&mut self, tid : Tid) ;
    fn pop(&self) -> Option<Tid>;
    fn tick(&self) -> bool;
    fn set_priority(&self, tid : Tid, priority : u8);
}

struct RRInfo {
    valid : bool,
    time : usize,
    prev : Tid,
    next : Tid,
}

pub struct RRScheduler {
    threads : Vec<RRInfo>,
    max_time : usize,
}

impl RRScheduler {
    pub fn new(max_time_slice : usize) -> Self {
        let rr = RRScheduler{
            threads : Vec::default(),
            max_time : max_time_slice,
        };
        rr.threads[0] = RRInfo {
            valid : false,
            time : 0,
            prev : 0,
            next : 0,
        };
        rr
    }
}

impl Scheduler for RRScheduler{
    pub fn push(&self, tid : Tid) {
        let tid = tid + 1;
        if tid > self.threads.len() {
            self.threads.resize_with(tid, ||{
                RRInfo {
                    valid : false,
                    time : 100,
                    prev : 0,
                    next : 0,
                }
            });
        }

        if self.threads[tid].time == 0 {
            self.threads[tid].time = self.max_time;
        }

        self.threads[tid].valid = true;
        self.threads[self.threads[0].prev].next = tid;
        self.threads[tid].prev = self.threads[0].prev;
        self.threads[0].prev = tid;
        self.threads[tid].next = 0;
    }

    fn pop(&self) -> Option<Tid> {
        let ret = self.thread[0].next;
        if ret != 0 {
            self.threads[self.threads[ret].next].prev = self.threads[ret].prev;
            self.threads[self.threads[ret].prev].next = self.threads[ret].next;
            self.threads[ret].prev = self.threads[ret].next = 0;
            self.threads[ret].valid = false;
            Some(ret)
        }else{
            None
        }
    }

    fn tick(&self) -> bool{
        let tid = self.threads[0].next;
        if self.threads[0].next != 0 {
            self.threads[self.threads[0].next].time -= 1;
            if self.threads[self.threads[0].next] == 0 {
                true
            }
        }
        true
    }

    fn set_priority(&self, tid : Tid, priority : u8) {

    }
}
