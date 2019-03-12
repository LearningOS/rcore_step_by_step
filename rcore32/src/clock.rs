pub static mut TICK: usize = 0; //类似ucore中的ticks的作用，用于时钟中断次数的计时，可以在实现分时的时候作为辅助。

use riscv::register::{sie, sstatus};
pub fn init() {
    unsafe{
        TICK = 0;
        sie::set_stimer();
    }
    clock_set_next_event();
    println!("++++setup timer !++++");
}

//获得当前的时间
use riscv::register::{time, timeh};
fn get_cycle() -> u64 {
    loop{
        let hi = timeh::read();
        let lo = time::read();
        let tmp = timeh::read();
        if(hi == tmp){
            return ((hi as u64)<<32) | (lo as u64);
        }
    }
}

//按照当前时刻加上频率的1%，设定下一次时钟中断发生的时间点
use crate::sbi::set_timer;
pub fn clock_set_next_event() {
    let timebase = 250000;
    set_timer(get_cycle() + timebase);
}
