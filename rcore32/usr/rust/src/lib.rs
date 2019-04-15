#![no_std]
#![feature(asm)]
#![feature(lang_items)]
#![feature(panic_info_message)]
#![feature(global_asm)]
#![feature(naked_functions)]

#[macro_use]
pub mod io;
pub mod syscall;
#[macro_use]
pub mod lang_items;

