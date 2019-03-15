use crate::interrupt::init as interrupt_init;
use crate::clock::init as clock_init;

global_asm!(include_str!("boot/entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    interrupt_init();
    clock_init();
    loop {}
}
