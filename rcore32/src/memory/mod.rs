mod frame_alloc;
mod paging;

use riscv::register::{satp,sstatus};
use riscv::paging::{PageTable, PageTableEntry, RecursivePageTable, PageTableFlags as Flags};
use riscv::addr::*;

use self::frame_alloc::{init as init_frame_allocator, test as test_frame_allocator, alloc_frame, dealloc_frame};
use self::paging::{InactivePageTable, active_table};
use crate::consts::{KERNEL_HEAP_SIZE, KERNEL_OFFSET , MAX_DTB_SIZE,
    MEMORY_OFFSET, PAGE_SIZE, MEMORY_END};
use crate::HEAP_ALLOCATOR;
use crate::context::TrapFrame;

use paging::ActivePageTable;

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

// Symbols provided by linker script
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
    let offset = KERNEL_OFFSET as usize - MEMORY_OFFSET as usize;
    println!("offset {:#x} ", offset);
    let mut inac_table = InactivePageTable::new();
    println!("stext {:#x} ", stext as usize);
    println!("etext {:#x} ", etext as usize);
    println!("sdata {:#x} ", sdata as usize);
    println!("edata {:#x} ", edata as usize);
    println!("srodata {:#x} ", srodata as usize);
    println!("erodata {:#x} ", erodata as usize);
    println!("sbss {:#x} ", sbss as usize);
    println!("ebss {:#x} ", ebss as usize);
    println!("start {:#x} ", start as usize);
    println!("end {:#x} ", end as usize);
    println!("bootstack {:#x} ", bootstack as usize);
    println!("bootstacktop {:#x} ", bootstacktop as usize);

    let mut idx = 0 as usize;
    let stext_start = stext as usize / PAGE_SIZE;
    let stext_end = (etext as usize - 1) / PAGE_SIZE + 1;
    idx = stext_start;
    while( idx <= stext_end) {
        inac_table.map(idx << 12, (idx << 12) - offset, Flags::EXECUTABLE | Flags::READABLE | Flags::VALID);
        idx += 1;
    }
    
    let srodata_start = srodata as usize / PAGE_SIZE;
    let srodata_end = (erodata as usize - 1) / PAGE_SIZE + 1;
    if(idx < srodata_start){ idx = srodata_start; }
    while( idx <= srodata_end) {
        inac_table.map(idx << 12, (idx << 12) - offset, Flags::READABLE | Flags::VALID);
        idx += 1;
    }
    
    let sdata_start = sdata as usize / PAGE_SIZE;
    let sdata_end = (edata as usize - 1) / PAGE_SIZE + 1;
    if(idx < sdata_start){ idx = sdata_start; }
    while( idx <= sdata_end) {
        inac_table.map(idx << 12, (idx << 12) - offset, Flags::WRITABLE | Flags::READABLE | Flags::VALID);
        idx += 1;
    }

    let bootstack_start = bootstack as usize / PAGE_SIZE;
    let bootstack_end = (bootstacktop as usize - 1) / PAGE_SIZE + 1;
    if(idx < bootstack_start){ idx = bootstack_start; }
    while( idx <= bootstack_end) {
        inac_table.map(idx << 12, (idx << 12) - offset, Flags::WRITABLE | Flags::READABLE | Flags::VALID);
        idx += 1;
    }

    let sbss_start = sbss as usize / PAGE_SIZE;
    let sbss_end = (ebss as usize - 1) / PAGE_SIZE + 1;
    if(idx < sbss_start){ idx = sbss_start; }
    while( idx <= sbss_end) {
        inac_table.map(idx << 12, (idx << 12) - offset, Flags::WRITABLE | Flags::READABLE | Flags::VALID);
        idx += 1;
    }

    let dtb_start = dtb as usize / PAGE_SIZE;
    let dtb_end = (dtb as usize + MAX_DTB_SIZE - 1) / PAGE_SIZE + 1;
    if(idx < dtb_start){ idx = dtb_start; }
    while( idx <= dtb_end) {
        inac_table.map(idx << 12, (idx << 12 )- offset, Flags::WRITABLE | Flags::READABLE | Flags::VALID);
        idx += 1;
    }

    InactivePageTable::print_p1(0x80ffd000);
    unsafe{
        inac_table.active();
    }
}

