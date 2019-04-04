#![feature(lang_items)]
#![feature(asm)]
#![feature(panic_info_message)]
#![feature(global_asm)]
#![feature(alloc)]
#![no_std]

#[macro_use]
mod io;
extern crate bitflags;
extern crate alloc;

mod lang_items;
mod sbi;
mod context;
mod interrupt;
mod init;
mod clock;
mod memory;
mod consts;
mod memory_set;

use buddy_system_allocator::LockedHeap;
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();
