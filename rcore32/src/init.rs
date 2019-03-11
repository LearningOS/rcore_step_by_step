use crate::interrupt::init as interrupt_init;

#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("Hello World");
    interrupt_init();
    unsafe{
        asm!("ebreak\n"::::);
        asm!("ebreak\n"::::);
    }
    println!("help !");
    loop {}
}
