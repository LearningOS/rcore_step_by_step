#![no_std]
#![feature(asm)]
#![feature(alloc)]
#![feature(lang_items)]
#![feature(panic_info_message)]
#![feature(linkage)]
#![feature(compiler_builtins_lib)]

extern crate alloc;

#[macro_use]
pub mod io;
pub mod lang_items;
pub mod syscall;


use buddy_system_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
