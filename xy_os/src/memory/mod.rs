mod frame_allocator;

use frame_allocator::{ init as init_frame_allocator, test as test_frame_allocator };
use crate::consts::*;
use crate::HEAP_ALLOCATOR;

pub fn init(dtb : usize) {
    use riscv::register::sstatus;
    unsafe {
        // Allow user memory access
        sstatus::set_sum();
    } 
    init_heap();
    if let Some((addr, mem_size)) = device_tree::DeviceTree::dtb_query_memory(dtb) {
        assert_eq!(addr, MEMORY_OFFSET);
        let KERNEL_END = dtb - KERNEL_OFFSET + MEMORY_OFFSET + PAGE_SIZE;
        let KERNEL_SIZE = KERNEL_END - addr;
        init_frame_allocator(KERNEL_END, KERNEL_SIZE);
    } else {
        panic!("failed to query memory");
    }
    test_frame_allocator();
}

fn init_heap() {
    static mut HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
    println!("heap init end");
}

pub enum PageFault{
    LoadPageFault,
    StorePageFault,
}

use crate::context::TrapFrame;
pub fn do_pgfault(tf: &mut TrapFrame, style: PageFault) {
    match style {
        PageFault::LoadPageFault => panic!("load pagefault"),
        PageFault::StorePageFault => panic!("store pagefault"),
    }
}
