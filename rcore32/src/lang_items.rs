use core::panic::PanicInfo;
use core::alloc::Layout;
/// This function is called on panic.


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
pub extern fn abort() {
    panic!("abort");
}
