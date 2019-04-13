use core::slice;
use core::mem::{size_of, size_of_val};

#[repr(C)]
pub struct SuperBlock {
    pub magic : u32,
    pub blocks : u32,
    pub unused_blocks : u32,
    pub info : u32,
    pub freemap_blocks : u32,
}

impl SuperBlock {
    pub fn check(&self) -> bool {
        println!("magic {:#x} ", self.magic);
        println!("blocks {}, unused_blocks {}, freemap_blocks {}",
                 self.blocks, self.unused_blocks, self.freemap_blocks);
        self.magic == MAGIC
    }
}

pub const MAGIC: u32 = 0x2f8dbe2a;
pub const BLKN_SUPER : usize = 0;

pub const BLKSIZE : usize = 1 << 12;

pub trait AsBuf {
    fn as_buf(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const _ as *const u8, size_of_val(self)) }
    }
    fn as_buf_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self as *mut _ as *mut u8, size_of_val(self)) }
    }
}

impl AsBuf for SuperBlock {}
impl AsBuf for DiskINode {}
