pub fn init(dtb : usize) {
    unsafe {
        sstatus::set_sum();
    } // Allow user memory access

    use riscv::register::satp;
    use riscv::paging::{PageTable, PageTableEntry};
    let page_table : *mut PageTable = 0xFF7FE000 as *mut PageTable;
    unsafe{
        for x in 0..5 {
            println!("ppn in pte is : {:#x} ", (*page_table)[0x300 + x].ppn() as usize);
        }
        println!("ppn in satp is : {:#x} ", (*page_table)[0x3fd].ppn() as usize);
        println!("ppn in satp is : {:#x} ", (*page_table)[0x3fe].ppn() as usize);
    }
    let t : *mut u32 = 0x1 as *mut u32;
    unsafe{
        *t = 1;
    }
}

pub enum PageFault{
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
}

use crate::context::TrapFrame;
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
