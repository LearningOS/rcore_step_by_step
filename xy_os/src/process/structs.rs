use crate::context::Context;
use crate::process::{ Tid, ExitCode };
use crate::consts::*;
use alloc::boxed::Box;
// use crate::memory::PageTable;

use xmas_elf::{
    header,
    program::{ Flags, SegmentData, Type },
    ElfFile,
};

#[derive(Clone)]
pub enum Status {
    Ready,
    Running(Tid),
    Sleeping,
    Exited(ExitCode),
}

pub struct Thread {
    context: Context, // 线程相关的上下文
    kstack: KernelStack, // 线程对应的内核栈
    pageTable: Option<KernelStack>,
}

use riscv::register::satp;
impl Thread {
    pub fn new_idle() -> Box<Thread> {
        unsafe {
            Box::new(Thread {
                context: Context::null(),
                kstack: KernelStack::new(),
                pageTable: None,
            })
        }
    }

    pub fn new_kernel(entry: extern "C" fn(usize) -> !, arg: usize) -> Box<Thread> {
        unsafe {
            let _kstack = KernelStack::new();
            Box::new(Thread {
                context: Context::new_kernel_thread(entry, arg, _kstack.top(), satp::read().bits()),
                kstack: _kstack,
                pageTable: None,
            })
        }
    }

    pub unsafe fn new_user(data: &[u8]) -> Box<Thread> 
    {
        let elf = ElfFile::new(data).expect("failed to read elf");
        let is32 = match elf.header.pt2 {
            header::HeaderPt2::Header32(_) => true,
            header::HeaderPt2::Header64(_) => false,
        };

        // Check ELF type
        match elf.header.pt2.type_().as_type() {
            header::Type::Executable => {println!("it really a elf");},
            header::Type::SharedObject => {},
            _ => panic!("ELF is not executable or shared object"),
        }

        // entry_point代表程序入口在文件中的具体位置
        let entry_addr = elf.header.pt2.entry_point() as usize;
        println!("entry: {:#x}", entry_addr);
        /*
        let mut vm = elf.make_memory_set(); // 为这个ｅｌｆ文件创建一个新的虚存系统，其中包含内核的地址空间和elf文件中程序的地址空间
        use crate::consts::{USER_STACK_OFFSET, USER_STACK_SIZE};
        let mut ustack_top = {  // 创建用户栈
            let (ustack_buttom, ustack_top) = (USER_STACK_OFFSET, USER_STACK_OFFSET + USER_STACK_SIZE);
            let paddr = alloc_frames(USER_STACK_SIZE / PAGE_SIZE).unwrap().start_address().as_usize();  // 这一行现在可以先留着，写文档暂时不用这一行
            vm.push(    // 创建一个内核栈之后还需要将这个内核栈装入虚存系统。
                ustack_buttom,
                ustack_top,
                MemoryAttr::new().set_user(),
                ByFrame::new(),
            );
            ustack_top
        };
        */
        let kstack = KernelStack::new();    //　为用户程序创建内核栈。用于线程切换
        let _pageTable = KernelStack::new_page();

        let entries = unsafe { &mut *(_pageTable.0 as *mut [u32; 1024]) };
        entries[KERNEL_OFFSET >> 22] = MEMORY_OFFSET as u32 >> 12 | 0xf; // VRWX

        let ustack_top = _pageTable.0 ;
        let _satp: usize = _pageTable.0 - KERNEL_OFFSET + MEMORY_OFFSET;
        let _satp: usize = (_satp >> 12) | (1 << 31);
        println!("{:#x}", _satp);
        Box::new(Thread{    // 注意下面创建上下文使用的是哪个栈
            context: Context::new_user_thread(entry_addr, ustack_top, kstack.top(), _satp),
            kstack: kstack,
            pageTable: Some(_pageTable),
        })
    }

    pub fn switch_to(&mut self, target: &mut Thread) {
        unsafe {
            self.context.switch(&mut target.context);
        }
    }
}

pub struct KernelStack(usize);
const STACK_SIZE: usize = 0x8000;

use alloc::alloc::{alloc, dealloc, Layout};
impl KernelStack {
    pub fn new() -> KernelStack {
        let bottom =
            unsafe {
                alloc(Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap())
            } as usize;
        KernelStack(bottom)
    }

    pub fn new_page() -> KernelStack {
        let bottom =
            unsafe {
                alloc(Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap())
            } as usize;
        // println!("{:#x}", bottom);
        KernelStack(bottom)
    }
    
    pub fn top(&self) -> usize {
        self.0 + STACK_SIZE
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.0 as _,
                Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap()
            );
        }
    }
}