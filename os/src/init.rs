global_asm!(include_str!("boot/entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    crate::interrupt::init();
    unsafe{
        asm!("ebreak"::::"volatile");
    }
    panic!("End of rust_main");
}
