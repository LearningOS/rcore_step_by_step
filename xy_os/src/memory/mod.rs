mod frame_allocator;

use frame_allocator::{ init as init_frame_allocator, test as test_frame_allocator };
use crate::consts::*;

pub fn init(dtb : usize) {
    use riscv::register::sstatus;
    unsafe {
        // Allow user memory access
        sstatus::set_sum();
    } 
    // let MEMORY_START: usize = (end as usize) - KERNEL_OFFSET + MEMORY_OFFSET + PAGE_SIZE;

    if let Some((addr, length)) = device_tree::DeviceTree::dtb_query_memory(dtb){
        assert_eq!(addr, MEMORY_OFFSET);
        init_frame_allocator(length, MEMORY_OFFSET, (dtb + MAX_DTB_SIZE) - KERNEL_OFFSET + MEMORY_OFFSET + PAGE_SIZE);
        println!("MemoryInfo : from {:#x}, length of region : {:#x}", addr , length);
    }else{
        println!("a null memory ?");
        panic!("failed to query memory");
    }
    test_frame_allocator();
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