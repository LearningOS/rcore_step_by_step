use bitmap_allocator::{BitAlloc4K, BitAlloc};
use crate::consts::*;
use lazy_static::*;
use spin::Mutex;
use riscv::addr::*;

// 物理页帧分配器
lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<BitAlloc4K> 
        = Mutex::new(BitAlloc4K::default());
}

pub fn init(start : usize, end : usize) {
    let page_start = (start - MEMORY_OFFSET) / PAGE_SIZE;
    let page_end = (end - MEMORY_OFFSET - 1) / PAGE_SIZE + 1;
    println!("{}", page_start);
    println!("{}", page_end);
    
    assert!(page_start < page_end, "illegal range for frame allocator");
    FRAME_ALLOCATOR.lock().insert(page_start..page_end);

    println!("++++init frame allocator succeed!++++");
}

pub fn alloc_frame() -> Option<Frame> {
    let ret = FRAME_ALLOCATOR
        .lock()
        .alloc()
        .map(|id| id * PAGE_SIZE + MEMORY_OFFSET);
    ret.map(|addr| Frame::of_addr(PhysAddr::new(addr)))
}

pub fn dealloc_frame(target: Frame) {
    FRAME_ALLOCATOR
        .lock()
        .dealloc(target.number() - MEMORY_OFFSET / PAGE_SIZE);
}

pub fn test() {
    let frame1: Frame = alloc_frame().expect("failed to alloc frame");
    println!("test frame_allocator : {:#x}", frame1.start_address().as_usize());
    let frame2: Frame = alloc_frame().expect("failed to alloc frame");
    println!("test frame_allocator : {:#x}", frame2.start_address().as_usize());
    dealloc_frame(frame1);
    let frame3: Frame = alloc_frame().expect("failed to alloc frame");
    println!("test frame_allocator : {:#x}", frame3.start_address().as_usize());
    dealloc_frame(frame2);
    dealloc_frame(frame3);
}