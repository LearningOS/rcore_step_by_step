use riscv::addr::*;
use crate::memory::frame_allocator::alloc_frame;
use riscv::asm::sfence_vma_all;

pub struct MemoryAttr(u32);

impl MemoryAttr {
    pub fn new() -> MemoryAttr {
        MemoryAttr(1)
    }

    pub fn set_readonly(mut self) -> MemoryAttr {
        self.0 = self.0 | 2; // 1 << 1
        self
    }

    pub fn set_execute(mut self) -> MemoryAttr {
        self.0 = self.0 | 8; // 1 << 3
        self
    }

    pub fn set_WR(mut self) -> MemoryAttr {
        self.0 = self.0 | 2 | 4;
        self
    }
}


fn get_PDX(addr: usize) -> usize {
    addr >> 22
}

fn get_PTX(addr: usize) -> usize {
    (addr >> 12) & 0x3ff
}

pub struct InactivePageTable {
    root_table: Frame,
    PDEs: [Option<Frame>; 1024],
    offset: usize,
}

impl InactivePageTable {
    pub fn new(_offset: usize) -> InactivePageTable {
        if let Some(_root_table) = alloc_frame() {
            return InactivePageTable {
                root_table: _root_table,
                PDEs: [None; 1024],
                offset: _offset,
            }
        } else {
            panic!("oom");
        }
    }
    
    fn pgtable_paddr(&mut self) -> usize {
        self.root_table.start_address().as_usize()
    }

    fn pgtable_vaddr(&mut self) -> usize {
        self.pgtable_paddr() + self.offset
    }

    pub fn set(&mut self, start: usize, end: usize, attr: MemoryAttr) {
        unsafe {

                println!("{}", attr.0);
            let mut vaddr = start & !0xfff; // 4K 对齐
            let pg_table = &mut *(self.pgtable_vaddr() as *mut [u32; 1024]);
            while vaddr < end {
                // 1-1. 通过页目录和 VPN[1] 找到所需页目录项
                let PDX = get_PDX(vaddr);
                let PDE = pg_table[PDX];
                // 1-2. 若不存在则创建
                if PDE == 0 {
                    self.PDEs[PDX] = alloc_frame();
                    let PDE_PPN = self.PDEs[PDX].unwrap().start_address().as_usize() >> 12;
                    pg_table[PDX] = (PDE_PPN << 10) as u32 | 0x1; // pointer to next level of page table
                }
                // 2. 页目录项包含了叶结点页表（简称页表）的起始地址，通过页目录项找到页表
                let pg_table_paddr = (pg_table[PDX] & (!0x3ff)) << 2;
                // 3. 通过页表和 VPN[0] 找到所需页表项
                // 4. 设置页表项包含的页面的起始物理地址和相关属性
                let pg_table_2 = &mut *((pg_table_paddr as usize + self.offset) as *mut [u32; 1024]);
                pg_table_2[get_PTX(vaddr)] = ((vaddr - self.offset) >> 2) as u32 | attr.0; // set XWRV
                vaddr += (1 << 12);
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
        Self::set_root_table((self.pgtable_paddr() >> 12) | (1 << 31));
        Self::flush_tlb();
    }
}
