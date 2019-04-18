use spin::Mutex;
use alloc::{ collections::VecDeque, };
use crate::process::{ Tid, current_tid, yield_now, wake_up};

#[derive(Default)]
pub struct Condvar {
    wait_queue : Mutex<VecDeque<Tid>>,
}

impl Condvar {
    pub fn new() -> Self {
        Condvar::default()
    }

    pub fn wait(&self) {
        let mut queue = self.wait_queue.lock();
        queue.push_back(current_tid());
        drop(queue);
        yield_now();
    }

    pub fn notify(&self) {
        let mut queue = self.wait_queue.lock();
        if let Some(tid) = queue.pop_front() {
            wake_up(tid);
            drop(queue);
            yield_now();
        }
    }
}
