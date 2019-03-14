use crate::interrupt::init as interrupt_init;

global_asm!(include_str!("boot/entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    interrupt_init();
    println!("Hello World");
    unsafe{
        asm!("ebreak\n"::::);
    }
    panic!("End of rust_main");
}
