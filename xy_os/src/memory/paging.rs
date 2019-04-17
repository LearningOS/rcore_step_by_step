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
    PTEs: [Option<Frame>; 1024],
    offset: usize,
}

impl InactivePageTable {
    pub fn new(_offset: usize) -> InactivePageTable {
        if let Some(_root_table) = alloc_frame() {
            return InactivePageTable {
                root_table: _root_table,
                PTEs: [None; 1024],
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
            println!("{:#x}", self.pgtable_vaddr());
            println!("{:#x}", self.pgtable_paddr());
            */
            //panic!("????");
            let mut addr = start & 0xffc00000; // 4K 对齐
            let pg_table = &mut *(self.pgtable_vaddr() as *mut [u32; 1024]);
            while addr < end {
                let PDX = get_PDX(addr);
                let PTE = pg_table[PDX];
                if (PTE == 0) {
                    self.PTEs[PDX] = alloc_frame();
                    let PPN = self.PTEs[PDX].unwrap().start_address().as_usize() >> 2;
                    pg_table[PDX] = PPN as u32 | 0x1; // pointer to next level of page table.
                }
                let PPN = (pg_table[PDX] & (!0x3ff)) << 2;
                let pg_table_2 = &mut *((PPN as usize + self.offset) as *mut [u32; 1024]);
                pg_table_2[get_PTX(addr)] = ((addr - self.offset) >> 2) as u32 | 0xf; // set XWRV
                addr += (1 << 12);
            }
        }
    }

    unsafe fn set_root_table(root_table: usize) { // 设置satp
        asm!("csrw satp, $0" :: "r"(root_table) :: "volatile");
    }

    unsafe fn flush_tlb() {
        sfence_vma_all();
    }

    pub unsafe fn activate(&mut self) {
        // println!()
        println!("{:#x}", (self.pgtable_paddr() >> 12) | (1 << 31));
        Self::set_root_table((self.pgtable_paddr() >> 12) | (1 << 31));
        Self::flush_tlb();
    }
}