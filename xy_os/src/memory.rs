pub fn init(dtb : usize) {
    use riscv::register::sstatus;
    unsafe {
        // Allow user memory access
        sstatus::set_sum();
    } 
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