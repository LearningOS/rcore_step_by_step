global_asm!(include_str!("boot/entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    let a = "Hello";
    let b = "World";
    println!("{}, {}!", a, b);
    panic!("End of rust_main");
}
