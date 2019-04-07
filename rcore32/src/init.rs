use crate::interrupt::init as interrupt_init;
use crate::clock::init as clock_init;
use crate::memory::{init as memory_init, clear_bss};
use crate::process::init as process_init;

#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn rust_main(hartid : usize, dtb : usize) -> ! {
    println!("Hello RISCV ! in hartid {}, dtb @ {:#x} ", hartid, dtb);

    interrupt_init();
    memory_init(dtb);
    process_init();
    clock_init();
    loop{}
}
