pub mod frame_allocator;

use crate::consts::*;
use crate::HEAP_ALLOCATOR;
use frame_allocator::{init as init_frame_allocator, test as test_frame_allocator};
use riscv::register::sstatus;

pub fn init() {
    unsafe {
        sstatus::set_sum(); // Allow user memory access
    }
    init_heap();
    let memory_start = (end as usize - KERNEL_OFFSET + MEMORY_OFFSET) + PAGE_SIZE;
    let memory_size = MEMORY_END - memory_start;
    init_frame_allocator(memory_start, memory_size);
    test_frame_allocator();
}

fn init_heap() {
    static mut HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

// Symbols provided by linker script
extern "C" {
    fn end();
}
