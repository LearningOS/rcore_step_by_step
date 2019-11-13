#![no_std]
#![feature(alloc)]
#![feature(asm)]
#![feature(alloc_error_handler)]
#![feature(global_asm)]
#![feature(naked_functions)]

#[macro_use]
mod io;

mod clock;
mod consts;
mod context;
mod init;
mod interrupt;
mod lang_items;
mod memory;
mod process;
mod sbi;

use buddy_system_allocator::LockedHeap;
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();
