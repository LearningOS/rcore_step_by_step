#![no_std]
#![no_main]

#[macro_use]
extern crate rcore32_user;

#[no_mangle]
pub fn main() {
    println!("Hello Fork !");
    for i in 0..3 {
        rcore32_user::syscall::sys_fork();
        println!("in thread {}, i : {}", rcore32_user::syscall::sys_getpid(), i);
    }
}
