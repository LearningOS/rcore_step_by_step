use alloc::collections::VecDeque;
use super::{Tid,};

pub enum Action {
    Wakeup(Tid),
}

pub struct Event {
    time : usize,
    data : Action,
}

pub struct Timer {
    events : VecDeque<Event>,
    tick : usize,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            events : VecDeque::new(),
            tick : 0,
        }
    }

    fn push(&mut self, action : Action, time : usize) {
    }

    fn tick(&self) -> Option<Event> {
        self.tick += 1;
    }
}
