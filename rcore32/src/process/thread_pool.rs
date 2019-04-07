use alloc::{ vec::Vec, boxed::Box};
use spin::Mutex;
use super::structs::*;
use super::timer::Timer;
use super::scheduler::Scheduler;


pub struct ThreadPool {
    threads: Vec<Mutex<Option<Thread>>>,    // 线程信号量的向量
    scheduler: Box<Scheduler>,      //　线程调度器
    timer: Mutex<Timer>,     // 时钟
}
