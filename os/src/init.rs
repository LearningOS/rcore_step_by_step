global_asm!(include_str!("boot/entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    crate::interrupt::init();
    crate::clock::init();
    crate::memory::init();
    loop {}
}
