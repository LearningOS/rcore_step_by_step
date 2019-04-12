use riscv::register::{
    sstatus::Sstatus,
    scause::Scause,
};

#[repr(C)]
pub struct TrapFrame {
    pub x: [usize; 32], // General registers
    pub sstatus: Sstatus, // Supervisor Status Register
    pub sepc: usize, // Supervisor exception program counter
    pub stval: usize, // Supervisor trap value
    pub scause: Scause, // Scause register: record the cause of exception/interrupt/trap
}

impl TrapFrame {
    pub fn increase_sepc(self: &mut Self) {
        self.sepc = self.sepc + 4;
    }
}

#[repr(C)]
pub struct Context {
    pub content_addr: usize // 上下文内容存储的位置
}

impl Context {
    #[naked]
    #[inline(never)]
    pub unsafe extern "C" fn switch(&mut self, target: &mut Context) {
        asm!(include_str!("process/switch.asm") :::: "volatile");
    }

    pub unsafe fn null() -> Context {
        Context { content_addr: 0 }  
    }

    pub unsafe fn new_kernel_thread(
        entry: extern "C" fn(usize) -> !,
        arg : usize,
        kstack_top : usize,
        satp : usize ) -> Context {
        ContextContent::new_kernel_thread(entry, arg, kstack_top, satp).push_at(kstack_top)
    }
}

#[repr(C)]
struct ContextContent {
    ra: usize, // 返回地址
    satp: usize, //　二级页表所在位置
    s: [usize; 12], // 被调用者保存的寄存器
}

use core::mem::zeroed;
impl ContextContent {
    fn new_kernel_thread(entry: extern "C" fn(usize) -> !, arg: usize , kstack_top: usize, satp: usize) -> ContextContent {
        let mut content: ContextContent = unsafe { zeroed() };
        content.ra = entry as usize;
        content.satp = satp;
        content.s[0] = arg;
        content
    }

    unsafe fn push_at(self, stack_top: usize) -> Context {
        let ptr = (stack_top as *mut ContextContent).sub(1);
        *ptr = self; // 拷贝 ContextContent
        Context { content_addr: ptr as usize }
    }
}
