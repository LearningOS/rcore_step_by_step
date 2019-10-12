use addr::*;
use core::ops::{Index, IndexMut};
use core::fmt::{Debug, Formatter, Error};

pub struct PageTable {
    entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
    /// Clears all entries.
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }

    /// Parameter `frame` is the actual physical frame where the root page table resides,
    ///  it can be anywhere in the main memory.
    /// Denote `recursive_index` by K, then virtual address of the root page table is
    ///  (K, K+1, 0) in Sv32, and (K, K, K+1, 0) in Sv39, and (K, K, K, K+1, 0) in Sv48.
    pub fn set_recursive(&mut self, recursive_index: usize, frame: Frame) {
        self[recursive_index].set(frame.clone(), EF::VALID);
        self[recursive_index + 1].set(frame.clone(), EF::VALID | EF::READABLE | EF::WRITABLE);
    }

    /// Setup identity map for the page with first level page table index.
    #[cfg(riscv32)]
    pub fn map_identity(&mut self, p2idx: usize, flags: PageTableFlags) {
        self.entries[p2idx].set(Frame::of_addr(PhysAddr::new(p2idx << 22)), flags);
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl Debug for PageTable {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_map()
            .entries(self.entries.iter().enumerate()
                .filter(|p| !p.1.is_unused()))
            .finish()
    }
}

#[derive(Copy, Clone)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }
    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.0)
    }
    pub fn ppn(&self) -> usize {
        self.0 >> 10
    }
    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.ppn() << 12)
    }
    pub fn frame(&self) -> Frame {
        Frame::of_addr(self.addr())
    }
    pub fn set(&mut self, frame: Frame, mut flags: PageTableFlags) {
        // U540 will raise page fault when accessing page with A=0 or D=0
        flags |= EF::ACCESSED | EF::DIRTY;
        self.0 = (frame.number() << 10) | flags.bits();
    }
    pub fn flags_mut(&mut self) -> &mut PageTableFlags {
        unsafe { &mut *(self as *mut _ as *mut PageTableFlags) }
    }
}

impl Debug for PageTableEntry {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("PageTableEntry")
            .field("frame", &self.frame())
            .field("flags", &self.flags())
            .finish()
    }
}

#[cfg(riscv64)]
const ENTRY_COUNT: usize = 1 << 9;
#[cfg(riscv32)]
const ENTRY_COUNT: usize = 1 << 10;

bitflags! {
    /// Possible flags for a page table entry.
    pub struct PageTableFlags: usize {
        const VALID =       1 << 0;
        const READABLE =    1 << 1;
        const WRITABLE =    1 << 2;
        const EXECUTABLE =  1 << 3;
        const USER =        1 << 4;
        const GLOBAL =      1 << 5;
        const ACCESSED =    1 << 6;
        const DIRTY =       1 << 7;
        const RESERVED1 =   1 << 8;
        const RESERVED2 =   1 << 9;
    }
}

type EF = PageTableFlags;
