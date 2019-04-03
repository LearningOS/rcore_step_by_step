mod bit_allocator;

use bit_allocator::{BitAlloc4K, BitAlloc, BitAlloc256, BitAllocCascade16};
use core::ops::Range;
use crate::consts::{MEMORY_OFFSET, PAGE_SIZE};

extern crate lazy_static;
use lazy_static::*;

use spin::Mutex;

use riscv::addr::*;

pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<Frame>;
}

pub trait FrameDeallocator {
    fn dealloc(&mut self, frame: Frame);
}

// 物理页帧分配器
lazy_static! {
    pub static ref FRAME_ALLOCATOR : Mutex<BitAlloc4K> 
        = Mutex::new(BitAlloc4K::default());
}

pub fn init(start : usize, end : usize) {
    let page_start = (start - MEMORY_OFFSET) / PAGE_SIZE;
    let page_end = (end - MEMORY_OFFSET - 1) / PAGE_SIZE + 1;
    assert!(page_start < page_end, "illegal range for frame allocator");
    let mut ba = FRAME_ALLOCATOR.lock();
    ba.insert(page_start..page_end);

    println!("frame allocator: init end");
}

pub fn test() {
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

#[derive(Debug, Clone, Copy)]
pub struct GlobalFrameAlloc;

impl FrameAllocator for GlobalFrameAlloc {
    fn alloc(&mut self) -> Option<Frame> {
        let ret = FRAME_ALLOCATOR
            .lock()
            .alloc()
            .map(|id| id * PAGE_SIZE + MEMORY_OFFSET);
        ret.map(|addr| Frame::of_addr(PhysAddr::new(addr)))
        // 这里在实现被动的页面换入换出的时候可能需要修改
    }
}

impl FrameDeallocator for GlobalFrameAlloc {
    fn dealloc(&mut self, target: Frame) {
        FRAME_ALLOCATOR
            .lock()
            .dealloc(target.number() - MEMORY_OFFSET / PAGE_SIZE);
    }
}

pub fn alloc_frame() -> Option<Frame> {
    GlobalFrameAlloc.alloc()
}
pub fn dealloc_frame(target: Frame) {
    GlobalFrameAlloc.dealloc(target);
}
