#![feature(lang_items)]
#![feature(asm)]
#![feature(panic_info_message)]
#![feature(global_asm)]
#![no_std]

#[macro_use]
mod io;
#[macro_use]
extern crate bitflags;

mod lang_items;
mod sbi;
mod context;
mod interrupt;
mod init;
mod clock;
mod memory;
mod consts;

use buddy_system_allocator::LockedHeap;
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();
