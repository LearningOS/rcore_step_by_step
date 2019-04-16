use core::panic::PanicInfo;
use core::alloc::Layout;
use crate::syscall::*;

#[linkage = "weak"]
#[no_mangle]
fn main() {
    panic!("No main() linked");
}

use crate::ALLOCATOR;
fn init_heap() {
    const HEAP_SIZE: usize = 0x1000;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe {
        ALLOCATOR.lock().init(HEAP.as_ptr() as usize, HEAP_SIZE);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let location = info.location().unwrap();
    let message = info.message().unwrap();
    println!(
        "\nPANIC in {} at line {} \n\t{}",
        location.file(),
        location.line(),
        message
    );

    loop {}
}

#[no_mangle]
pub extern "C" fn _start(_argc: isize, _argv: *const *const u8) -> ! {
    init_heap();
    main();
    sys_exit(0)
}

#[no_mangle]
pub extern fn abort() {
    panic!("abort");
}

#[lang = "oom"]
fn oom(_: Layout) -> ! {
    panic!("out of memory");
}

#[lang = "eh_personality"]
fn eh_personality() {}

