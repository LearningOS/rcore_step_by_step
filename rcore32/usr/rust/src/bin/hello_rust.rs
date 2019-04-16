#![no_std]
#![no_main]

#[macro_use]
extern crate rcore32_user;

#[no_mangle]
pub fn main() {
    println!("Hello RISCV !");
    println!("Hello RISCV !");
    //rcore32_user::syscall::sys_exit(0);
}
