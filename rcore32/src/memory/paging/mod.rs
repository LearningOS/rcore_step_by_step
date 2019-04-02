use core::slice;
use crate::consts::{RECURSIVE_INDEX, PAGE_SIZE};
use riscv::asm::{sfence_vma, sfence_vma_all};
use riscv::paging::{
    Mapper, PageTable, PageTableEntry, PageTableFlags as EF, 
    RecursivePageTable,FrameAllocator, FrameDeallocator,
};
use riscv::register::satp;
use riscv::addr::*;
use super::frame_alloc::{alloc_frame, dealloc_frame};

const TEMP_PAGE_ADDR: usize = 0xcafeb000;    // 临时挂靠的地址
const ROOT_PAGE_TABLE: *mut PageTable =
    ((RECURSIVE_INDEX << 12 << 10) | ((RECURSIVE_INDEX + 1) << 12)) as *mut PageTable;

pub struct PageEntry(&'static mut PageTableEntry, Page);

pub struct ActivePageTable(RecursivePageTable<'static>, PageEntry);

impl PageEntry {
    fn update(&mut self) {
        unsafe {
            sfence_vma(0, self.1.start_address().as_usize());
        }
    }

    fn accessed(&self) -> bool {
        self.0.flags().contains(EF::ACCESSED)
    }
    fn clear_accessed(&mut self) {
        self.0.flags_mut().remove(EF::ACCESSED);
    }

    fn dirty(&self) -> bool {
        self.0.flags().contains(EF::DIRTY)
    }
    fn clear_dirty(&mut self) {
        self.0.flags_mut().remove(EF::DIRTY);
    }

    fn writable(&self) -> bool {
        self.0.flags().contains(EF::WRITABLE)
    }
    fn set_writable(&mut self, value: bool) {
        self.0.flags_mut().set(EF::WRITABLE, value);
    }

    fn present(&self) -> bool {
        self.0.flags().contains(EF::VALID | EF::READABLE)
    }
    fn set_present(&mut self, value: bool) {
        self.0.flags_mut().set(EF::VALID | EF::READABLE, value);
    }

    fn target(&self) -> usize {
        self.0.addr().as_usize()
    }
    fn set_target(&mut self, target: usize) {
        let flags = self.0.flags();
        let frame = Frame::of_addr(PhysAddr::new(target));
        self.0.set(frame, flags);
    }

    fn user(&self) -> bool {
        self.0.flags().contains(EF::USER)
    }
    fn set_user(&mut self, value: bool) {
        self.0.flags_mut().set(EF::USER, value);
    }

    fn execute(&self) -> bool {
        self.0.flags().contains(EF::EXECUTABLE)
    }
    fn set_execute(&mut self, value: bool) {
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

    fn unmap(&mut self, addr: usize) {
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
        active_table().with_temporary_map(target, |_, table : &mut PageTable|{
            table.zero();
            table.set_recursive(RECURSIVE_INDEX, frame.clone());
        });
        InactivePageTable{ root_frame : frame }
    }

    pub fn map(&mut self, addr: usize, target: usize, flags : EF) {
        let paddr = PhysAddr::new(target);  // 物理地址
        let vaddr = VirtAddr::new(addr);    // 虚拟地址

        let p1_addr = active_table().with_temporary_map(self.root_frame.start_address(), |_, table : &mut PageTable|{
            if table[vaddr.p2_index()].is_unused() {
                let new_frame : Frame = alloc_frame().expect("failed to alloc frame");
                table[vaddr.p2_index()].set(new_frame, EF::VALID);
                let ret = new_frame.start_address();
                ret
            }else{
                let ret = table[vaddr.p2_index()].addr();
                ret
            }
        });
        
        active_table().with_temporary_map(p1_addr, |_, table : &mut PageTable|{
            table[vaddr.p1_index()].set(Frame::of_addr(paddr), flags);
        });
    }

    pub fn unmap(&mut self, addr : usize ) {
    }

    pub unsafe fn active(&self) {
        let old_token = Self::active_token();
        let new_token = self.token();
        println!("switch table {:x?} -> {:x?}", old_token, new_token);
        if old_token != new_token {
            Self::set_token(new_token);
            Self::flush_tlb();
        }
    }

    fn token(&self) -> usize {
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
        active_table().with_temporary_map(self.root_frame.start_address(), |_, table : &mut PageTable|{
            let mut idx = 0;
            while idx < 1024{
                println!("{:#x}, {:#x}", idx, table[idx].ppn());
                idx += 1;
            }
        });
    }

    pub fn print_p1(addr : usize) {
        active_table().with_temporary_map(PhysAddr::new(addr), |_, table : &mut PageTable|{
            let mut idx = 0;
            while idx < 1024{
                println!("{:#x}, {:#x}", idx, table[idx].ppn());
                idx += 1;
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
