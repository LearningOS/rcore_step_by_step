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
