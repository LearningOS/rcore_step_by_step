use crate::process::Tid;
use RoundRobinScheduler::RRScheduler;

pub struct Scheduler {
    scheduler: RRScheduler,
}

impl Scheduler {
    pub fn new(max_time_slice: usize) -> Scheduler {
        let s = Scheduler {
            scheduler: RRScheduler::new(max_time_slice),
        };
        s
    }

    pub fn push(&mut self, tid: Tid) {
        self.scheduler.push(tid);
    }

    pub fn pop(&mut self) -> Option<Tid> {
        self.scheduler.pop()
    }

    pub fn tick(&mut self) -> bool {
        self.scheduler.tick()
    }

    pub fn exit(&mut self, tid: Tid) {
        self.scheduler.exit(tid);
    }
}
