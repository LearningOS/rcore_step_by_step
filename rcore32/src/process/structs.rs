use alloc::{ sync::Arc, boxed::Box, collections::BTreeMap, string::String};
pub use crate::context::{ Context, TrapFrame};
use super::{KernelStack, Tid, ExitCode, Pid};
use crate::memory::{ current_root,);
use crate::memory_set::{ MemorySet, handler::ByFrame, attr::MemoryAttr};
use crate::fs::{ file_handle::FileHandle, ROOT_INODE};
use rcore_fs::vfs::{ INode, };
use xmas_elf::{
    header,
    program::{Flags, SegmentData, Type},
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
    pub context : Context,  // 线程相关的上下文
    pub kstack : KernelStack,   // 线程对应的内核栈
    pub proc : Option<Box<Process>>,  // 线程对应的进程
}

impl Thread {
    pub unsafe fn new_init() -> Box<Thread> {
        Box::new(Thread {
            context : Context::null(),
            kstack : KernelStack::new(),
            proc : None,
        })
    }

    pub unsafe fn new_kernel(entry : extern "C" fn(usize) -> !, arg : usize) -> Box<Thread> {
        let kstack = KernelStack::new();
        Box::new(Thread {
            context : Context::new_kernel_thread(entry, arg, kstack.top(), current_root()) ,
            kstack : kstack,
            proc : None,
        })
    }

    pub unsafe fn new_user(data : &[u8]) -> Box<Thread> {
        let elf = ElfFile::new(data).expect("failed to read elf");

        // Check ELF type
        match elf.header.pt2.type_().as_type() {
            header::Type::Executable => {println!("it really a elf");},
            header::Type::SharedObject => {},
            _ => panic!("ELF is not executable or shared object"),
        }

        // entry_point代表程序入口在文件中的具体位置
        let entry_addr = elf.header.pt2.entry_point() as usize;
        println!("entry : {:#x}", entry_addr);

        let mut vm = elf.make_memory_set(); // 为这个ｅｌｆ文件创建一个新的虚存系统，其中包含内核的地址空间和elf文件中程序的地址空间
        use crate::consts::{USER_STACK_OFFSET, USER_STACK_SIZE};
        let mut ustack_top = {  // 创建用户栈
            let (ustack_buttom, ustack_top) = (USER_STACK_OFFSET, USER_STACK_OFFSET + USER_STACK_SIZE);
            vm.push(    // 创建一个内核栈之后还需要将这个内核栈装入虚存系统。
                ustack_buttom,
                ustack_top,
                MemoryAttr::new().set_user(),
                ByFrame::new(),
            );
            ustack_top
        };

        let kstack = KernelStack::new();    //　为用户程序创建内核栈。用于线程切换
        Box::new(Thread{    // 注意下面创建上下文使用的是哪个栈
            context : Context::new_user_thread(entry_addr, ustack_top, kstack.top(), vm.token()),
            kstack : kstack,
            proc : Some(Box::new(Process{
                pid : None,
                vm,
                files : BTreeMap::new(),
                cwd : String::from("/"),
            })),
        })
    }

    pub unsafe fn fork(&mut self, tf : &mut TrapFrame) -> Box<Thread> {
        let proc = self.proc.as_mut().unwrap();
        let kstack = KernelStack::new();
        let vm : MemorySet = proc.vm.clone();
        let cwd = proc.cwd.clone();
        let files = proc.files.clone();
        //println!("vm token : {}", vm.token());
        Box::new(Thread{
            context : Context::new_fork(tf, kstack.top(), vm.token()),
            kstack,
            proc : Some(Box::new(Process{
                pid : None,
                vm,
                files,
                cwd,
            })),
        })
    }

    pub unsafe fn switch_to(&mut self, target : &mut Thread) {
        self.context.switch(&mut target.context);
    }
}

pub struct Process {
    pid : Option<Pid>,
    vm : MemorySet,
    pub files : BTreeMap<usize, FileHandle>,
    cwd : String,
}

pub const FOLLOW_MAX_DEPTH : usize = 1;

impl Process {
    // 根据文件描述符返回文件句柄
    pub fn get_file_handle(&mut self, fd : usize) -> Option<&mut FileHandle> {
        self.files.get_mut(&fd)
    }

    // 根据路径名称查找inode
    pub fn lookup_inode(&self, path: &str) -> Option<Arc<INode>> {
        if let Ok(current) = ROOT_INODE.lookup(&self.cwd) {
            if let Ok(inode) = current.lookup_follow(path, FOLLOW_MAX_DEPTH) {
                return Some(inode);
            }
        }
        None
    }

    pub fn get_free_fd(&self) -> usize {
        (0..).find(|i| !self.files.contains_key(i)).unwrap()
    }
}

trait ElfExt {
    fn make_memory_set(&self) -> MemorySet;
}

use core::str;
impl ElfExt for ElfFile<'_> {
    fn make_memory_set(&self) -> MemorySet {
        println!("creating MemorySet from ELF");
        let mut ms = MemorySet::new_kern(); // 创建自带内核地址空间的虚拟存储系统

        for ph in self.program_iter() { // 枚举文件中的程序段
            if ph.get_type() != Ok(Type::Load) {
                continue;
            }
            // 获取程序段的大小和起始地址(虚拟的)
            let virt_addr = ph.virtual_addr() as usize;
            let mem_size = ph.mem_size() as usize;
            // 将数据读取为ｕ８的数组
            let data = match ph.get_data(self).unwrap() {
                SegmentData::Undefined(data) => data,
                _ => unreachable!(),
            };

            // Get target slice
            let target = {  // 可以看到，这里的virt_addr是根据文件中的虚拟地址得到的，所以target应该仅用于with函数中
                println!("virt_addr {:#x}, mem_size {:#x}", virt_addr, mem_size);
                ms.push(
                    virt_addr,
                    virt_addr + mem_size,
                    ph.flags().to_attr(),
                    ByFrame::new(),
                );
                unsafe { ::core::slice::from_raw_parts_mut(virt_addr as *mut u8, mem_size) }
            };
            // Copy data
            unsafe {
                ms.with(|| {    // with函数的作用是，将当前这个未激活页表激活并执行一个函数，然后切换回原来的页表
                    if data.len() != 0 {
                        target[..data.len()].copy_from_slice(data);
                    }
                    target[data.len()..].iter_mut().for_each(|x| *x = 0);
                });
            }
        }
        ms
    }
}

trait ToMemoryAttr {
    fn to_attr(&self) -> MemoryAttr;
}

impl ToMemoryAttr for Flags {
    fn to_attr(&self) -> MemoryAttr {   // 将文件中各个段的读写权限转换为页表权限
        let mut flags = MemoryAttr::new().set_user();
        if self.is_execute() {
            flags = flags.set_execute();
        }
        flags
    }
}
