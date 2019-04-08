#![feature(lang_items)]
#![feature(asm)]
#![feature(panic_info_message)]
#![feature(global_asm)]
#![no_std]
#![feature(alloc)]

#[macro_use]
pub mod io;

extern crate alloc;

mod lang_items;
mod context;
mod interrupt;
mod init;
mod clock;
mod memory;
mod consts;
