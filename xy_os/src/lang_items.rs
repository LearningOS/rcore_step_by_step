use core::panic::PanicInfo;
use core::alloc::Layout;

// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

#[no_mangle]
pub extern fn abort() {
    panic!("abort!");
}

#[lang = "oom"]
fn oom(_: Layout) -> ! {
    panic!("out of memory");
}