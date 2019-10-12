#![no_std]
#![feature(asm)]
#![feature(alloc_error_handler)]
#![feature(global_asm)]

#[macro_use]
mod io;

mod init;
mod lang_items;
mod sbi;
mod context;
mod interrupt;
mod clock;
mod memory;
mod consts;

use buddy_system_allocator::LockedHeap;
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();
