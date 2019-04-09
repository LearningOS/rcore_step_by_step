use riscv::register::{scause::Scause, sstatus, sstatus::Sstatus};

#[derive(Clone)]
#[repr(C)]
pub struct TrapFrame {
    pub x: [usize; 32], // general registers
    pub sstatus: Sstatus, // Supervisor Status Register
    pub sepc: usize, // Supervisor exception program counter, save the trap virtual address (here is used to save the process program entry addr?)
    pub stval: usize, // Supervisor trap value
    pub scause: Scause, // scause register: record the cause of exception/interrupt/trap
}

impl TrapFrame {
    pub fn increase_sepc(self: &mut Self) {
        self.sepc = self.sepc + 4;
    }

    pub fn print_trapframe(self: &mut Self) {
        println!("print the trapfram:  
                    cause : {:#x} 
                    sepc : {:#x} 
                    stval : {:#x} ",
                    self.scause.bits(),
                    self.sepc ,
                    self.stval as usize);
    }
}

#[repr(C)]
struct ContextContent {
    ra : usize, // 返回地址
    satp : usize, //　二级页表所在位置
    s : [usize; 12], // 被调用者保存的寄存器
    tf : TrapFrame, // 中断帧
}

extern "C" {
    fn trap_return();
}

use core::mem::zeroed;
impl ContextContent {
    fn new_kernel_thread(entry : extern "C" fn(usize) -> !, arg : usize , kstack_top : usize, satp : usize) -> Self {
        ContextContent{
            ra : trap_return as usize,
            satp,
            s : [0;12],
            tf : {
                let mut tf: TrapFrame = unsafe { zeroed() };
                tf.x[10] = arg; // 存放第一个参数的寄存器a0
                tf.x[2] = kstack_top;   // 栈顶ｓｐ
                tf.sepc = entry as usize;   // sepc在调用sret之后将被被赋值给ＰＣ
                tf.sstatus = sstatus::read();
                tf.sstatus.set_spie(true);
                tf.sstatus.set_sie(false);
                tf.sstatus.set_spp(sstatus::SPP::Supervisor);   // 代表sret之后的特权级仍为Ｓ
                tf
            },
        }
    }

    unsafe fn push_at(self, stack_top : usize) -> Context {
        let ptr = (stack_top as *mut Self).sub(1); //real kernel stack top
        *ptr = self;
        Context { sp: ptr as usize }
    }
}

#[derive(Debug)]
pub struct Context {
    sp : usize // 上下文内容存储的位置
}

impl Context {
    #[naked]
    #[inline(never)]
    pub unsafe extern "C" fn switch(&mut self, _target : &mut Self) {
        asm!(
            r"
        .equ XLENB, 4
        .macro Load reg, mem
            lw \reg, \mem
        .endm
        .macro Store reg, mem
            sw \reg, \mem
        .endm"
        );
        // 请注意下面汇编中对a0以及a1中的值的使用和处理。这表明switch函数的调用将会改变它的参数指向的内存中存储的数据。
        asm!("
        // save from's registers
        addi  sp, sp, (-XLENB*14)
        Store sp, 0(a0)
        Store ra, 0*XLENB(sp)
        Store s0, 2*XLENB(sp)
        Store s1, 3*XLENB(sp)
        Store s2, 4*XLENB(sp)
        Store s3, 5*XLENB(sp)
        Store s4, 6*XLENB(sp)
        Store s5, 7*XLENB(sp)
        Store s6, 8*XLENB(sp)
        Store s7, 9*XLENB(sp)
        Store s8, 10*XLENB(sp)
        Store s9, 11*XLENB(sp)
        Store s10, 12*XLENB(sp)
        Store s11, 13*XLENB(sp)
        csrr  s11, satp
        Store s11, 1*XLENB(sp)

        // restore to's registers
        Load sp, 0(a1)
        Load s11, 1*XLENB(sp)
        csrw satp, s11
        Load ra, 0*XLENB(sp)
        Load s0, 2*XLENB(sp)
        Load s1, 3*XLENB(sp)
        Load s2, 4*XLENB(sp)
        Load s3, 5*XLENB(sp)
        Load s4, 6*XLENB(sp)
        Load s5, 7*XLENB(sp)
        Load s6, 8*XLENB(sp)
        Load s7, 9*XLENB(sp)
        Load s8, 10*XLENB(sp)
        Load s9, 11*XLENB(sp)
        Load s10, 12*XLENB(sp)
        Load s11, 13*XLENB(sp)
        addi sp, sp, (XLENB*14)

        Store zero, 0(a1)
        ret"
        : : : : "volatile" )
    }

    pub const unsafe fn null() -> Self {
        Context { sp : 0 }  
    }

    pub unsafe fn new_kernel_thread(
        entry: extern "C" fn(usize) -> !,
        arg : usize,
        kstack_top : usize,
        satp : usize
    ) -> Self {
        ContextContent::new_kernel_thread(entry, arg, kstack_top, satp).push_at(kstack_top)
    }
}
