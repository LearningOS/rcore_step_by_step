use crate::consts::RECURSIVE_INDEX;
use riscv::asm::{sfence_vma, sfence_vma_all};
use riscv::paging::{
    Mapper, PageTable as RvPageTable, PageTableEntry, PageTableFlags as EF, 
    RecursivePageTable,FrameAllocator, FrameDeallocator,
};
use riscv::register::satp;
use riscv::addr::*;
use super::frame_alloc::{alloc_frame, dealloc_frame};


const ROOT_PAGE_TABLE: *mut RvPageTable =
    ((RECURSIVE_INDEX << 12 << 10) | ((RECURSIVE_INDEX + 1) << 12)) as *mut RvPageTable;

pub struct PageEntry(&'static mut PageTableEntry, Page);

pub struct ActivePageTable(RecursivePageTable<'static>, PageEntry);

#[derive(Debug)]
pub struct InactivePageTable {
    root_frame: Frame,
}

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
        // use riscv::paging:Mapper::map_to,
        // map the 4K `page` to the 4K `frame` with `flags`
        let flags = EF::VALID | EF::READABLE | EF::WRITABLE;
        let page = Page::of_addr(VirtAddr::new(addr));
        let frame = Frame::of_addr(PhysAddr::new(target));
        // map the page to the frame using FrameAllocatorForRiscv
        // we may need frame allocator to alloc frame for new page table(first/second)
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
}

impl InactivePageTable {

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
