#![no_std]
#![no_main]

#[macro_use]
extern crate rcore32_user;

#[no_mangle]
pub fn main() {
    loop{
        rcore32_user::syscall::sys_sleep(100);
        println!("hello sleep");
    }
}
