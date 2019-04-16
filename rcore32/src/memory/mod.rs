pub mod frame_alloc;
pub mod paging;

use riscv::register::sstatus;
use self::frame_alloc::{init as init_frame_allocator, test as test_frame_allocator,};
use crate::consts::{KERNEL_HEAP_SIZE, KERNEL_OFFSET , MAX_DTB_SIZE,
                MEMORY_OFFSET, PAGE_SIZE,};
use crate::HEAP_ALLOCATOR;
use crate::context::TrapFrame;
use crate::drivers::device_tree::dtb_query_memory;

pub fn init(dtb : usize) {
    println!("+------------ to initialize memory ------------+");
    unsafe {
        sstatus::set_sum();
    } // Allow user memory access

    init_heap();

    if let Some((addr, length)) = dtb_query_memory(dtb){
        assert_eq!(addr, MEMORY_OFFSET);
        init_frame_allocator(length, MEMORY_OFFSET, (dtb + MAX_DTB_SIZE) - KERNEL_OFFSET + MEMORY_OFFSET + PAGE_SIZE);
        println!("MemoryInfo : from {:#x}, length of region : {:#x}", addr , length);
    }else{
        println!("a null memory ?");
        panic!("failed to query memory");
    }

    test_frame_allocator();

    remap_kernel(dtb);
    // 执行clear_bss之后会导致failed to alloc frame问题，还没有找出原因，所以暂时不执行clear_bss()
    //unsafe{
        //clear_bss();
    //}
    println!("+---------- now memory is initialized ----------+");
}

pub enum PageFault{
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
}

pub fn do_pgfault(tf : &mut TrapFrame, style : PageFault) {
    tf.print_trapframe();
    match style {
        PageFault::InstructionPageFault => panic!("A instruction pagefault"),
        PageFault::LoadPageFault => panic!("A load pagefault"),
        PageFault::StorePageFault => panic!("A store pagefault"),
    }
}

pub unsafe fn clear_bss() {
    let start = sbss as usize;
    let end = ebss as usize;
    let step = core::mem::size_of::<usize>();
    for i in (start..end).step_by(step) {
        (i as *mut usize).write(0);
    }
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

use core::mem;
fn remap_kernel(dtb : usize) {
    let offset = - ( KERNEL_OFFSET as isize - MEMORY_OFFSET as isize);

    use crate::memory_set::{MemorySet, handler::Linear, attr::MemoryAttr};
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
    mem::forget(memset);
}

use riscv::register::satp;
pub fn current_root() -> usize {
    satp::read().bits()
}
