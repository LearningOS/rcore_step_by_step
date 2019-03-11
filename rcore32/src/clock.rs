use riscv::register::sie;
pub fn init() {
    unsafe{
        sie::set_stimer();
    }
    clock_set_next_event();
    println!("setup timer !");
}

use riscv::register::{time, timeh};
fn get_cycle() -> u64 {
    loop{
        let hi = timeh::read();
        let lo = time::read();
        let tmp = timeh::read();
        if(hi == tmp){
            return ((hi as u64)<<32 | (lo as u64));
        }
    }
}

use crate::sbi::set_timer;
fn clock_set_next_event() {
    let timebase = 250000;
    set_timer(get_cycle() + timebase);
}
