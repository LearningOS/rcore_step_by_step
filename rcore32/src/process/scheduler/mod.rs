use super::Tid;
use alloc::{collections::VecDeque, vec::Vec,};

pub trait Scheduler {
    fn push(&mut self, tid : Tid) ;
    fn pop(&mut self) -> Option<Tid>;
    fn tick(&mut self) -> bool;
    fn set_priority(&self, tid : Tid, priority : u8);
}

#[derive(Default)]
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
        let mut rr = RRScheduler{
            threads : Vec::default(),
            max_time : max_time_slice,
        };
        rr.threads.push(RRInfo {
            valid : false,
            time : 0,
            prev : 0,
            next : 0,
        });
        rr
    }
}

impl Scheduler for RRScheduler{
    fn push(&mut self, tid : Tid) {
        let tid = tid + 1;
        //println!("tid {}, len {}", tid, self.threads.len());
        if tid + 1 > self.threads.len() {
            self.threads.resize_with(tid + 1, Default::default);
        }
        //println!("tid {}, len {}", tid, self.threads.len());

        //println!("tid {}, time {}", tid, self.threads[tid].time);
        if self.threads[tid].time == 0 {
            self.threads[tid].time = self.max_time;
        }

        let prev = self.threads[0].prev;
        self.threads[tid].valid = true;
        self.threads[prev].next = tid;
        self.threads[tid].prev = prev;
        self.threads[0].prev = tid;
        self.threads[tid].next = 0;
    }

    fn pop(&mut self) -> Option<Tid> {
        let ret = self.threads[0].next;
        if ret != 0 {
            let next = self.threads[ret].next;
            let prev = self.threads[ret].prev;
            self.threads[next].prev = prev;
            self.threads[prev].next = next;
            self.threads[ret].prev = 0;
            self.threads[ret].next = 0;
            self.threads[ret].valid = false;
            Some(ret-1)
        }else{
            None
        }
    }

    fn tick(&mut self) -> bool{
        let tid = self.threads[0].next;
        if tid != 0 {
            self.threads[tid].time -= 1;
            if self.threads[tid].time == 0 {
                println!("tick a 0, the tid is {}", tid);
                return true;
            }
        }else{
            return true;
        }
        false
    }

    fn set_priority(&self, tid : Tid, priority : u8) {

    }
}
