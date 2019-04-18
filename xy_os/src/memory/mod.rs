pub mod frame_allocator;
mod paging;

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
    // test_frame_allocator();
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
    println!("remaping");
    let offset = KERNEL_OFFSET as usize - MEMORY_OFFSET as usize;
    use crate::memory::paging::{ InactivePageTable, MemoryAttr };
    let mut pg_table = InactivePageTable::new(offset);
    pg_table.set(stext as usize, etext as usize, MemoryAttr::new().set_readonly().set_execute());
    pg_table.set(sdata as usize, edata as usize, MemoryAttr::new().set_WR());
    pg_table.set(srodata as usize, erodata as usize, MemoryAttr::new().set_readonly());
    pg_table.set(sbss as usize, ebss as usize, MemoryAttr::new().set_WR());
    pg_table.set(bootstack as usize, bootstacktop as usize, MemoryAttr::new().set_WR());
    pg_table.set(dtb, dtb + MAX_DTB_SIZE, MemoryAttr::new().set_WR());
    unsafe {
        pg_table.activate();
    }
}
