mod frame_allocator;

use frame_allocator::{ init as init_frame_allocator, test as test_frame_allocator };
use crate::consts::*;
use crate::HEAP_ALLOCATOR;
use crate::drivers::device_tree::dtb_query_memory;

pub fn init(dtb : usize) {
    use riscv::register::sstatus;
    unsafe {
        // Allow user memory access
        sstatus::set_sum();
    } 
    // let MEMORY_START: usize = (end as usize) - KERNEL_OFFSET + MEMORY_OFFSET + PAGE_SIZE;
    init_heap();
    println!("?>>");
    if let Some((addr, length)) = dtb_query_memory(dtb) {
        assert_eq!(addr, MEMORY_OFFSET);
        println!("MemoryInfo : from {:#x}, length of region : {:#x}", addr , length);
        init_frame_allocator(length, MEMORY_OFFSET, (dtb + MAX_DTB_SIZE) - KERNEL_OFFSET + MEMORY_OFFSET + PAGE_SIZE);
    } else {
        println!("a null memory ?");
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

#[allow(dead_code)]
extern "C" {
    fn stext();
    fn etext();
    fn sdata();
    fn edata();
    fn srodata();
    fn erodata();
    fn sbss();
    fn ebss();
    fn start();
    fn end();
    fn bootstack();
    fn bootstacktop();
}