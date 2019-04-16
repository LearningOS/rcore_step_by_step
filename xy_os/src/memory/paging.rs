use riscv::addr::*;
use crate::memory::frame_allocator::alloc_frame;
use riscv::asm::sfence_vma_all;

pub struct MemoryAttr(u32);

impl MemoryAttr {
    pub fn new() -> MemoryAttr {
        MemoryAttr(1)
    }

    pub fn clear(mut self) -> MemoryAttr {
        self.0 = 0;
        self
    }

    pub fn set_valid(mut self) -> MemoryAttr {
        self.0 = self.0 | 1; // 1 << 0
        self
    }

    pub fn set_readonly(mut self) -> MemoryAttr {
        self.0 = self.0 | 2; // 1 << 1
        self
    }

    pub fn set_execute(mut self) -> MemoryAttr {
        self.0 = self.0 | 8; // 1 << 3
        self
    }

    pub fn set_all(mut self) -> MemoryAttr {
        self.0 = self.0 | 1 | 2 | 4 | 8;
        self
    }

    pub fn is_valid(&self) -> bool {
        (self.0 & 1) == 1
    }
}


fn get_PDX(addr: usize) -> usize {
    addr >> 22
}

fn get_PTX(addr: usize) -> usize {
    (addr >> 12) & 0x3ff
}

fn get_PTE(pg_dir: usize, addr: usize) -> usize {
    unsafe {
        let pg_table = &mut *(pg_dir as *mut [u32; 1024]);
        pg_table[get_PDX(addr)] as usize
    }
}

fn get_leaf_PTE(PTE: usize, addr: usize) -> usize {
    1
}

pub struct InactivePageTable {
    root_table: Frame,
    offset: usize,
}

impl InactivePageTable {
    pub fn new(_offset: usize) -> InactivePageTable {
        if let Some(_root_table) = alloc_frame() {
            return InactivePageTable {
                root_table: _root_table,
                offset: _offset,
            }
        } else {
            panic!("oom");
        }
    }

    pub fn attr(&self) -> MemoryAttr {
        MemoryAttr((self.root_table.start_address().as_usize() as u32) & 1023)
    }

    fn pgtable_paddr(&mut self) -> usize {
        self.root_table.start_address().as_usize()
    }

    fn pgtable_vaddr(&mut self) -> usize {
        self.pgtable_paddr() + self.offset
    }

    pub fn set(&mut self, start: usize, end: usize, attr: MemoryAttr) {
        unsafe {
            /*
            println!("{:#x}", start);
            println!("{:#x}", end);
            println!("{}", end - start);
            */
            let mut addr = start;
            let pg_table = &mut *(self.pgtable_vaddr() as *mut [u32; 1024]);
            while addr < end {
                pg_table[get_PDX(addr)] = (addr - self.offset) as u32 | attr.0;
                addr += (1 << 22);
            }
        }
    }

    unsafe fn set_root_table(root_table: usize) { // 设置satp。切换二级页表
        asm!("csrw satp, $0" :: "r"(root_table) :: "volatile");
    }

    unsafe fn flush_tlb() {
        sfence_vma_all();
    }

    pub unsafe fn activate(&mut self) {
        // println!()
        println!("{:#x}", (self.pgtable_paddr() >> 10) | (1 << 31));
        Self::set_root_table((self.pgtable_paddr() >> 10) | (1 << 31));
        Self::flush_tlb();
    }
}