use super::Tid;

pub trait Scheduler {
    fn push(&mut self, tid : Tid) ;
    fn pop(&self) -> Tid;
    fn tick(&self) -> bool;
    fn set_priority(&self, tid : Tid, priority : u8);
}
