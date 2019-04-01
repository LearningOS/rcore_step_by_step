use crate::interrupt::init as interrupt_init;
use crate::clock::init as clock_init;
use crate::memory::init as memory_init;

global_asm!(include_str!("boot/entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main(hartid : usize, dtb : usize) -> ! {
    interrupt_init();
    println!("Hello RISCV ! in hartid {}, dtb @ {:#x} ", hartid, dtb);
    memory_init(dtb);
    clock_init();
    let x: *mut u32 = 0xc0020020 as *mut u32;
    unsafe {
        println!("{:#x}", *x);
    }
    loop {}
}
