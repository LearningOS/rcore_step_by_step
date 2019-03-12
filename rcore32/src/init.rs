use crate::interrupt::init as interrupt_init;
use crate::clock::init as clock_init;
use crate::sbi::other_ecall as ecall;

#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("Hello World");
    interrupt_init();
    clock_init();
    loop{}
}
