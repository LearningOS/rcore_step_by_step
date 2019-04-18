#![no_std]
#![no_main]

#[macro_use]
extern crate rcore32_user;

#[no_mangle]
pub fn main() {
    println!("Hello RISCV And Rust!");
    let s = "rust/hello_fork\0";
    rcore32_user::syscall::sys_exec(s.as_ptr());
    let s1 = "rust/hello_sleep\0";
    rcore32_user::syscall::sys_exec(s1.as_ptr());
}
