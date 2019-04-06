mod bit_allocator;
mod buddy_alloc;

use buddy_alloc::{BuddyAlloc, log2_down};
use crate::consts::{MEMORY_OFFSET, PAGE_SIZE};

extern crate lazy_static;
use lazy_static::*;
use spin::Mutex;
use riscv::addr::*;

// 物理页帧分配器
lazy_static! {
    pub static ref BUDDY_ALLOCATOR : Mutex<BuddyAlloc> 
        = Mutex::new(BuddyAlloc::new());
}

pub fn init(mem_size : usize, start : usize, end : usize) {
    let page_start = (start - MEMORY_OFFSET) / PAGE_SIZE;
    let page_end = (end - MEMORY_OFFSET - 1) / PAGE_SIZE + 1;
    assert!(page_start < page_end, "illegal range for frame allocator");

    let mut bu = BUDDY_ALLOCATOR.lock();
    println!("{}",log2_down(mem_size / PAGE_SIZE) as u8); 
    bu.init(log2_down(mem_size / PAGE_SIZE) as u8);
    bu.alloc(page_end - page_start);
    println!("frame allocator: init end");
}

pub fn test() { // 这个测试还应该更复杂一点，TODO
    let frame1 : Frame = alloc_frame().expect("failed to alloc frame");
    println!("test frame_allocator : {:#x}" , frame1.start_address().as_usize());
    let frame2 : Frame = alloc_frame().expect("failed to alloc frame");
    println!("test frame_allocator : {:#x}" , frame2.start_address().as_usize());
    dealloc_frame(frame1);
    let frame3 : Frame = alloc_frame().expect("failed to alloc frame");
    println!("test frame_allocator : {:#x}" , frame3.start_address().as_usize());
    dealloc_frame(frame2);
    dealloc_frame(frame3);
}

pub fn alloc_frame() -> Option<Frame> {
    alloc_frames(1)
}
pub fn dealloc_frame(target: Frame) {
    dealloc_frames(target,1);
}
pub fn alloc_frames(size : usize) -> Option<Frame> {
    let ret = BUDDY_ALLOCATOR
        .lock()
        .alloc(size)
        .map(|id| id * PAGE_SIZE + MEMORY_OFFSET);
    ret.map(|addr| Frame::of_addr(PhysAddr::new(addr)))
}
pub fn dealloc_frames(target : Frame, size : usize) {
    BUDDY_ALLOCATOR
        .lock()
        .dealloc(target.number() - MEMORY_OFFSET / PAGE_SIZE, size);
}
