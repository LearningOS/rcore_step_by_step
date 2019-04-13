use alloc::{ sync::Arc, boxed::Box};
pub use crate::context::Context;
use super::{KernelStack, Tid, ExitCode};
use crate::memory::{ current_root, frame_alloc::alloc_frames);
use crate::memory_set::{ MemorySet, handler::ByFrame, attr::MemoryAttr};
use crate::consts::PAGE_SIZE;

use spin::Mutex;
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
    pub proc : Option<Arc<Mutex<Process>>>,  // 线程对应的进程
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

    pub unsafe fn new_user(data : &[u8]) -> Box<Thread> 
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

        if let Ok(load_path) = elf.get_interpreter() {
            println!("loader specified as {}", &load_path);
        }

        let entry_addr = elf.header.pt2.entry_point() as usize;
        println!("entry : {:#x}", entry_addr);

        let mut vm = elf.make_memory_set();
        use crate::consts::{USER_STACK_OFFSET, USER_STACK_SIZE};
        let mut ustack_top = {
            let (ustack_buttom, ustack_top) = (USER_STACK_OFFSET, USER_STACK_OFFSET + USER_STACK_SIZE);
            let paddr = alloc_frames(USER_STACK_SIZE / PAGE_SIZE).unwrap().start_address().as_usize();
            vm.push(
                ustack_buttom,
                ustack_top,
                MemoryAttr::new().set_user(),
                ByFrame::new(),
            );
            ustack_top
        };

        let kstack = KernelStack::new();
        Box::new(Thread{
            context : Context::new_user_thread(entry_addr, ustack_top, kstack.top(), vm.token()),
            kstack : kstack,
            proc : None,
        })
    }

    pub unsafe fn switch_to(&mut self, target : &mut Thread) {
        self.context.switch(&mut target.context);
    }
}

pub struct Process {
}

trait ElfExt {
    fn get_interpreter(&self) -> Result<&str, &str>;

    fn make_memory_set(&self) -> MemorySet;
}

use core::str;
impl ElfExt for ElfFile<'_> {
    fn get_interpreter(&self) -> Result<&str, &str> {
        let header = self
            .program_iter()
            .filter(|ph| ph.get_type() == Ok(Type::Interp))
            .next()
            .ok_or("no interp header")?;
        let mut data = match header.get_data(self)? {
            SegmentData::Undefined(data) => data,
            _ => unreachable!(),
        };
        // skip NULL
        while let Some(0) = data.last() {
            data = &data[..data.len() - 1];
        }
        let path = str::from_utf8(data).map_err(|_| "failed to convert to utf8")?;
        Ok(path)
    }

    fn make_memory_set(&self) -> MemorySet {
        println!("creating MemorySet from ELF");
        let mut ms = MemorySet::new_kern();

        for ph in self.program_iter() {
            if ph.get_type() != Ok(Type::Load) {
                continue;
            }
            let virt_addr = ph.virtual_addr() as usize;
            let mem_size = ph.mem_size() as usize;
            let data = match ph.get_data(self).unwrap() {
                SegmentData::Undefined(data) => data,
                _ => unreachable!(),
            };

            // Get target slice
            let target = {
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
                ms.with(|| {
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
    fn to_attr(&self) -> MemoryAttr {
        let mut flags = MemoryAttr::new().set_user();
        if self.is_execute() {
            flags = flags.set_execute();
        }
        flags
    }
}
