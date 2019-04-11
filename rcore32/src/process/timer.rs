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
    time : usize,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            events : VecDeque::new(),
            time : 0,
        }
    }

    pub fn push(&mut self, action : Action, time_after : usize) {
        let time = self.time + time_after;
        let event = Event{ time : time , data : action, };
        let mut it = self.events.iter();
        let mut i: usize = 0;
        loop {
            match it.next() {
                None => break,
                Some(e) if e.time >= time => break,
                _ => {}
            }
            i += 1;
        }
        self.events.insert(i, event);
    }

    pub fn pop(&mut self) -> Option<Action> {
        match self.events.front() {
            None => return None,
            Some(event) if event.time != self.time => return None,
            _ => {}
        };
        self.events.pop_front().map(|t| t.data)
    }

    pub fn tick(&mut self) {
        self.time += 1;
        if let Some(timer) = self.events.front() {
            let current_time = self.time;
            println!("current time {}, timer {}", current_time, timer.time);
        }
    }
}
