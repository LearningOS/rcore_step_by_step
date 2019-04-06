use crate::memory::paging::ActivePageTable;
use super::attr::MemoryAttr;
use core::fmt::Debug;
use alloc::boxed::Box;


pub trait MemoryHandler : Debug + 'static{
    fn box_clone(&self) -> Box<MemoryHandler>;
    fn map(&self, pt : &mut ActivePageTable, addr : usize, attr : &MemoryAttr); 
}

impl Clone for Box<MemoryHandler> {
    fn clone(&self) -> Box<MemoryHandler> {
        self.box_clone()
    }
}

#[derive(Debug,Clone)]
pub struct Linear {
    offset : isize,
}

impl MemoryHandler for Linear {
    fn box_clone(&self) -> Box<MemoryHandler> {
        Box::new(self.clone())
    }

    fn map(&self, pt : &mut ActivePageTable, addr : usize, attr : &MemoryAttr) {
        attr.apply(pt.map(addr, (addr as isize + self.offset) as usize));
    }

}

impl Linear {
    pub fn new(off : isize) -> Self {
        Linear{
            offset : off,
        }
    }
}
                   

