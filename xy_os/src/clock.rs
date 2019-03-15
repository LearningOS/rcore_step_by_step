pub static mut TICK: usize = 0;

use riscv::register::sie;
pub fn init() {
    unsafe{
        TICK = 0;
        sie::set_stimer();
    }
    clock_set_next_event();
    println!("++++setup timer !++++");
}

use riscv::register::{time, timeh};
fn get_cycle() -> u64 {
    loop {
        let hi = timeh::read();
        let lo = time::read();
        let tmp = timeh::read();
        if (hi == tmp) {
            return ((hi as u64) << 32) | (lo as u64);
        }
    }
}

static timebase: u64 = 100000;
use bbl::sbi::set_timer;
pub fn clock_set_next_event() {
    set_timer(get_cycle() + timebase);
}