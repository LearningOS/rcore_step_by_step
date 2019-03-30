use crate::interrupt::init as interrupt_init;
use crate::clock::init as clock_init;
use crate::sbi::other_ecall as ecall;
use crate::memory::init as memory_init;

#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn rust_main(hartid : usize, dtb : usize) -> ! {
    println!("Hello RISCV ! in hartid {}, dtb @ {:#x} ", hartid, dtb);
    interrupt_init();
    memory_init(dtb);
    clock_init();
    loop{}
}
