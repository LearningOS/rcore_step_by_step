use crate::consts::{RECURSIVE_INDEX, PAGE_SIZE};
use riscv::asm::{sfence_vma, sfence_vma_all};
use riscv::paging::{
    Mapper, PageTable as RvPageTable, PageTableEntry, PageTableFlags as EF, 
    RecursivePageTable,FrameAllocator, FrameDeallocator,
};
use riscv::register::satp;
use riscv::addr::*;
use super::frame_alloc::{alloc_frame, dealloc_frame};

const TEMP_PAGE_ADDR: usize = 0xcafeb000;    // 临时挂靠的地址
const ROOT_PAGE_TABLE: *mut RvPageTable =
    ((RECURSIVE_INDEX << 12 << 10) | ((RECURSIVE_INDEX + 1) << 12)) as *mut RvPageTable;

pub struct PageEntry(&'static mut PageTableEntry, Page);

pub struct ActivePageTable(RecursivePageTable<'static>, PageEntry);

impl PageEntry {
    pub fn update(&mut self) {
        unsafe {
            sfence_vma(0, self.1.start_address().as_usize());
        }
    }

    pub fn accessed(&self) -> bool {
        self.0.flags().contains(EF::ACCESSED)
    }
    pub fn clear_accessed(&mut self) {
        self.0.flags_mut().remove(EF::ACCESSED);
    }

    pub fn dirty(&self) -> bool {
        self.0.flags().contains(EF::DIRTY)
    }
    pub fn clear_dirty(&mut self) {
        self.0.flags_mut().remove(EF::DIRTY);
    }

    pub fn writable(&self) -> bool {
        self.0.flags().contains(EF::WRITABLE)
    }
    pub fn set_writable(&mut self, value: bool) {
        self.0.flags_mut().set(EF::WRITABLE, value);
    }

    pub fn present(&self) -> bool {
        self.0.flags().contains(EF::VALID | EF::READABLE)
    }
    pub fn set_present(&mut self, value: bool) {
        self.0.flags_mut().set(EF::VALID | EF::READABLE, value);
    }

    pub fn target(&self) -> usize {
        self.0.addr().as_usize()
    }
    pub fn set_target(&mut self, target: usize) {
        let flags = self.0.flags();
        let frame = Frame::of_addr(PhysAddr::new(target));
        self.0.set(frame, flags);
    }

    pub fn user(&self) -> bool {
        self.0.flags().contains(EF::USER)
    }
    pub fn set_user(&mut self, value: bool) {
        self.0.flags_mut().set(EF::USER, value);
    }

    pub fn execute(&self) -> bool {
        self.0.flags().contains(EF::EXECUTABLE)
    }
    pub fn set_execute(&mut self, value: bool) {
        self.0.flags_mut().set(EF::EXECUTABLE, value);
    }

}

impl ActivePageTable {
    pub unsafe fn new() -> Self {
        ActivePageTable(
            RecursivePageTable::new(&mut *ROOT_PAGE_TABLE).unwrap(),
            ::core::mem::uninitialized(),
        )
    }

    pub fn map(&mut self, addr: usize, target: usize) -> &mut PageEntry {
        let flags = EF::VALID | EF::READABLE | EF::WRITABLE;
        let page = Page::of_addr(VirtAddr::new(addr));
        let frame = Frame::of_addr(PhysAddr::new(target));
        self.0
            .map_to(page, frame, flags, &mut FrameAllocatorForRiscv)
            .unwrap()
            .flush();
        self.get_entry(addr).expect("fail to get entry")
    }

    pub fn unmap(&mut self, addr: usize) {
        let page = Page::of_addr(VirtAddr::new(addr));
        let (_, flush) = self.0.unmap(page).unwrap();
        flush.flush();
    }

    fn get_entry(&mut self, vaddr: usize) -> Option<&mut PageEntry> {   // 类似get_pte
        let page = Page::of_addr(VirtAddr::new(vaddr));
        if let Ok(e) = self.0.ref_entry(page.clone()) {
            let e = unsafe { &mut *(e as *mut PageTableEntry) };
            self.1 = PageEntry(e, page);
            Some(&mut self.1 as &mut PageEntry)
        } else {
            None
        }
    }

    fn with_temporary_map<T, D>(
        &mut self,
        target: PhysAddr,   // 挂靠的页表
        f: impl FnOnce(&mut Self, &mut D) -> T, // 进行的操作
    ) -> T {
        self.map(TEMP_PAGE_ADDR, target.as_usize());
        let data =
            unsafe { &mut *(self.get_page_slice_mut(VirtAddr::new(TEMP_PAGE_ADDR)).as_ptr() as *mut D) };
        let ret = f(self, data);
        self.unmap(TEMP_PAGE_ADDR);
        ret
    }   

    fn get_page_slice_mut<'a>(&mut self, addr: VirtAddr) -> &'a mut [u8] {
        unsafe { core::slice::from_raw_parts_mut((addr.as_usize() & !(PAGE_SIZE - 1)) as *mut u8, PAGE_SIZE) }
    }
}

pub fn active_table() -> ActivePageTable {
    unsafe{ ActivePageTable::new() }
}

#[derive(Debug)]
pub struct InactivePageTable {
    root_frame: Frame,
}

impl InactivePageTable {

    pub fn new() -> Self {
        let frame = alloc_frame().expect("InactivePageTable new : failed to alloc frame");
        let target = PhysAddr::new(frame.start_address().as_usize());
        active_table().with_temporary_map(target, |_, table : &mut RvPageTable|{
            table.zero();
            table.set_recursive(RECURSIVE_INDEX, frame.clone());
        });
        InactivePageTable{ root_frame : frame }
    }

    pub fn map(&mut self, addr: usize, target: usize, flags : EF) {
        self.edit(|pt : &mut ActivePageTable|{
            if pt.get_entry(addr).is_none() {
                let entry = pt.map(addr, target);
                entry.set_execute(flags.contains(EF::EXECUTABLE));
                entry.set_writable(flags.contains(EF::WRITABLE));
            }
        })
    }

    pub fn unmap(&mut self, addr : usize ) {
        self.edit(|pt : &mut ActivePageTable|{
            pt.unmap(addr)
        })
    }

    pub unsafe fn activate(&self) {
        let old_token = Self::active_token();
        let new_token = self.token();
        //println!("switch table {:x?} -> {:x?}", old_token, new_token);
        if old_token != new_token {
            Self::set_token(new_token);
            Self::flush_tlb();
        }
    }

    pub fn token(&self) -> usize {
        self.root_frame.number() | (1 << 31) // as satp
    }

    unsafe fn set_token(token: usize) { // 设置satp。切换二级页表
        asm!("csrw satp, $0" :: "r"(token) :: "volatile");
    }

    fn active_token() -> usize {    // 返回正在运行的二级页表的起始地址
        satp::read().bits()
    }

    fn flush_tlb() {    // 刷新tlb
        unsafe {
            sfence_vma_all();
        }
    }

    pub fn print_table(&mut self) {
        active_table().with_temporary_map(self.root_frame.start_address(), |_, table : &mut RvPageTable|{
            let mut idx = 0;
            while idx < 1024{
                println!("{:#x}, {:#x}", idx, table[idx].ppn());
                idx += 1;
            }
        });
    }

    pub fn print_p1(addr : usize) {
        active_table().with_temporary_map(PhysAddr::new(addr), |_, table : &mut RvPageTable|{
            let mut idx = 0;
            while idx < 1024{
                println!("{:#x}, {:#x}", idx, table[idx].ppn());
                idx += 1;
            }
        });
    }

    pub fn edit<T>(&mut self, f : impl FnOnce(&mut ActivePageTable) -> T) -> T {
        let target = satp::read().frame().start_address();
        active_table().with_temporary_map(target, |active_table, root_table : &mut RvPageTable|{
            let backup = root_table[RECURSIVE_INDEX].clone();

            root_table[RECURSIVE_INDEX].set(self.root_frame.clone(), EF::VALID);
            unsafe {
                sfence_vma_all();
            }

            let ret = f(active_table);  // 此时的f运行在新的上下文中，即active_table代表的是现在这个InactivePageTable

            root_table[RECURSIVE_INDEX] = backup;
            unsafe {
                sfence_vma_all();
            }

            ret
        })
    }

    pub unsafe fn with<T>(&self, f: impl FnOnce() -> T) -> T {
        let old_token = Self::active_token();
        let new_token = self.token();
        //println!("switch table {:x?} -> {:x?}", old_token, new_token);
        if old_token != new_token {
            Self::set_token(new_token);
            Self::flush_tlb();
        }
        let ret = f();
        //println!("switch table {:x?} -> {:x?}", new_token, old_token);
        if old_token != new_token {
            Self::set_token(old_token);
            Self::flush_tlb();
        }
        ret
    }

    pub fn map_kernel(&mut self) {
        let table = unsafe { &mut *ROOT_PAGE_TABLE };
        extern "C" {
            fn start();
            fn end();
        }
        let mut entrys: [PageTableEntry; 16] = unsafe { core::mem::uninitialized() };
        let entry_start = start as usize >> 22;
        let entry_end = (end as usize >> 22) + 1;
        let entry_count = entry_end - entry_start;
        for i in 0..entry_count {
            entrys[i] = table[entry_start + i];
        }

        self.edit(|_| {
            // NOTE: 'table' now refers to new page table
            for i in 0..entry_count {
                table[entry_start + i] = entrys[i];
            }
        });
    }
}

struct FrameAllocatorForRiscv;

impl FrameAllocator for FrameAllocatorForRiscv {
    fn alloc(&mut self) -> Option<Frame> {
        alloc_frame()
    }
}

impl FrameDeallocator for FrameAllocatorForRiscv {
    fn dealloc(&mut self, frame: Frame) {
        dealloc_frame(frame);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct PageRange {
    start: usize,
    end: usize,
}

impl Iterator for PageRange {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.start < self.end {
            let page = self.start << 12;
            self.start += 1;
            Some(page)
        } else {
            None
        }
    }
}

impl PageRange {
    pub fn new(start_addr : usize, end_addr : usize) -> Self {
        PageRange{
            start : start_addr / PAGE_SIZE,
            end : (end_addr - 1) / PAGE_SIZE + 1,
        }
    }
}
