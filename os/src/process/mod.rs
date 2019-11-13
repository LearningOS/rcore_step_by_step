mod structs;
use structs::Thread;

pub fn init() {
    let mut loop_thread = Thread::new_idle();
    let mut hello_thread = Thread::new_kernel(hello_thread, 666);
    loop_thread.switch_to(&mut hello_thread);
}

#[no_mangle]
pub extern "C" fn hello_thread(arg: usize) -> ! {
    println!("hello thread");
    println!("arg is {}", arg);
    loop {}
}
