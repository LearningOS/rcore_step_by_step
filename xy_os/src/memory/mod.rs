pub mod frame_allocator;
pub mod paging;

use frame_allocator::{ init as init_frame_allocator, test as test_frame_allocator };
use crate::consts::*;
use crate::HEAP_ALLOCATOR;

pub fn init(dtb: usize) {
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
    remap_kernel(dtb);
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

extern "C" {
    // text
    fn stext();
    fn etext();
    // data
    fn sdata();
    fn edata();
    // read only
    fn srodata();
    fn erodata();
    // bss
    fn sbss();
    fn ebss();
    // kernel
    fn start();
    fn end();
    // boot
    fn bootstack();
    fn bootstacktop();
}

fn remap_kernel(dtb: usize) {
    let offset = - ( KERNEL_OFFSET as isize - MEMORY_OFFSET as isize);
    use crate::memory_set::MemorySet;
    let mut memset = MemorySet::new();
    memset.push(
        stext as usize,
        etext as usize,
        MemoryAttr::new().set_execute().set_readonly(),
        Linear::new(offset),
    );
    memset.push(
        srodata as usize,
        erodata as usize,
        MemoryAttr::new().set_readonly(),
        Linear::new(offset),
    );
    memset.push(
        sdata as usize,
        edata as usize,
        MemoryAttr::new(),
        Linear::new(offset),
    );
    memset.push(
        bootstack as usize,
        bootstacktop as usize,
        MemoryAttr::new(),
        Linear::new(offset),
    );
    memset.push(
        sbss as usize,
        ebss as usize,
        MemoryAttr::new(),
        Linear::new(offset),
    );
    memset.push(
        dtb as usize,
        dtb as usize + MAX_DTB_SIZE,
        MemoryAttr::new(),
        Linear::new(offset),
    );
    unsafe{
        memset.activate();
    }
}
