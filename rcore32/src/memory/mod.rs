mod frame_alloc;
pub mod paging;

use riscv::register::sstatus;
use riscv::paging::{PageTable, PageTableFlags as Flags};

use self::frame_alloc::{init as init_frame_allocator, test as test_frame_allocator,};
use self::paging::InactivePageTable;
use crate::consts::{KERNEL_HEAP_SIZE, KERNEL_OFFSET , MAX_DTB_SIZE,
    MEMORY_OFFSET, PAGE_SIZE, MEMORY_END};
use crate::HEAP_ALLOCATOR;
use crate::context::TrapFrame;

const TEMP_ADDRESS : *mut PageTable= 0xFFC00000 as *mut PageTable;

pub fn init(dtb : usize) {
    unsafe {
        sstatus::set_sum();
    } // Allow user memory access

    let page_table : *mut PageTable = 0xFF7FE000 as *mut PageTable;
    unsafe{
        for x in 0..5 {
            println!("ppn of pte {} in pte is : {:#x} ", x, (*page_table)[0x300 + x].ppn() as usize);
        }
    }

    init_frame_allocator((end as usize) - KERNEL_OFFSET + MEMORY_OFFSET + PAGE_SIZE , MEMORY_END);
    test_frame_allocator();

    init_heap();

    remap_kernel(dtb);
    // 执行clear_bss之后会导致failed to alloc frame问题，还没有找出原因，所以暂时不执行clear_bss()
    //unsafe{
        //clear_bss();
    //}
    test_frame_allocator();

    println!("hello world");

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
}
