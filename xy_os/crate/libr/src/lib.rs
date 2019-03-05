#![no_std]
#![feature(alloc)]
#![feature(lang_items)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::alloc::Layout;
pub mod io;

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
#[lang = "omm"]
fn omm(_: Layout) -> ! {
    panic!("omm");
}