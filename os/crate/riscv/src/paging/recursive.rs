use super::frame_alloc::*;
use super::page_table::{*, PageTableFlags as F};
use addr::*;

pub trait Mapper {
    /// Creates a new mapping in the page table.
    ///
    /// This function might need additional physical frames to create new page tables. These
    /// frames are allocated from the `allocator` argument. At most three frames are required.
    fn map_to(&mut self, page: Page, frame: Frame, flags: PageTableFlags, allocator: &mut impl FrameAllocator) -> Result<MapperFlush, MapToError>;

    /// Removes a mapping from the page table and returns the frame that used to be mapped.
    ///
    /// Note that no page tables or pages are deallocated.
    fn unmap(&mut self, page: Page) -> Result<(Frame, MapperFlush), UnmapError>;

    /// Get the reference of the specified `page` entry
    fn ref_entry(&mut self, page: Page) -> Result<&mut PageTableEntry, FlagUpdateError>;

    /// Updates the flags of an existing mapping.
    fn update_flags(&mut self, page: Page, flags: PageTableFlags) -> Result<MapperFlush, FlagUpdateError> {
        self.ref_entry(page).map(|e| {
            *e.flags_mut() = flags;
            MapperFlush::new(page)
        })
    }

    /// Return the frame that the specified page is mapped to.
    fn translate_page(&mut self, page: Page) -> Option<Frame> {
        match self.ref_entry(page) {
            Ok(e) => if e.is_unused() { None } else { Some(e.frame()) },
            Err(_) => None,
        }
    }

    /// Maps the given frame to the virtual page with the same address.
    fn identity_map(&mut self, frame: Frame, flags: PageTableFlags, allocator: &mut impl FrameAllocator) -> Result<MapperFlush, MapToError>
    {
        let page = Page::of_addr(VirtAddr::new(frame.start_address().as_usize()));
        self.map_to(page, frame, flags, allocator)
    }
}

#[must_use = "Page Table changes must be flushed or ignored."]
pub struct MapperFlush(Page);

impl MapperFlush {
    /// Create a new flush promise
    fn new(page: Page) -> Self {
        MapperFlush(page)
    }

    /// Flush the page from the TLB to ensure that the newest mapping is used.
    pub fn flush(self) {
        unsafe { crate::asm::sfence_vma(0, self.0.start_address().as_usize()); }
    }

    /// Don't flush the TLB and silence the “must be used” warning.
    pub fn ignore(self) {}
}

/// This error is returned from `map_to` and similar methods.
#[derive(Debug)]
pub enum MapToError {
    /// An additional frame was needed for the mapping process, but the frame allocator
    /// returned `None`.
    FrameAllocationFailed,
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of an already mapped huge page.
    ParentEntryHugePage,
    /// The given page is already mapped to a physical frame.
    PageAlreadyMapped,
}

/// An error indicating that an `unmap` call failed.
#[derive(Debug)]
pub enum UnmapError {
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of a huge page and can't be freed individually.
    ParentEntryHugePage,
    /// The given page is not mapped to a physical frame.
    PageNotMapped,
    /// The page table entry for the given page points to an invalid physical address.
    InvalidFrameAddress(PhysAddr),
}

/// An error indicating that an `update_flags` call failed.
#[derive(Debug)]
pub enum FlagUpdateError {
    /// The given page is not mapped to a physical frame.
    PageNotMapped,
}

struct TempMap<'a> {
    entry: &'a mut PageTableEntry,
    pt_addr: VirtAddr,
}

impl<'a> TempMap<'a> {
    #[cfg(riscv32)]
    unsafe fn new(rec_idx: usize) -> Self {
        TempMap {
            entry: VirtAddr::from_page_table_indices(rec_idx, rec_idx + 1, (rec_idx + 2) * 4).as_mut(),
            pt_addr: VirtAddr::from_page_table_indices(rec_idx, rec_idx + 2, 0),
        }
    }
    #[cfg(riscv64)]
    unsafe fn new(rec_idx: usize, type_: PageTableType) -> Self {
        let p4_idx = match type_ {
            PageTableType::Sv39 => if rec_idx >> 8 == 0 { 0o000 } else { 0o777 },
            PageTableType::Sv48 => rec_idx,
            _ => panic!("invalid page table type"),
        };
        TempMap {
            entry: VirtAddr::from_page_table_indices(p4_idx, rec_idx, rec_idx, rec_idx + 1, (rec_idx + 2) * 8).as_mut(),
            pt_addr: VirtAddr::from_page_table_indices(p4_idx, rec_idx, rec_idx, rec_idx + 2, 0),
        }
    }
    fn map(&mut self, frame: Frame) -> &mut PageTable {
        self.entry.set(frame, F::VALID | F::READABLE | F::WRITABLE);
        unsafe { crate::asm::sfence_vma(0, self.pt_addr.as_usize()); }
        unsafe { self.pt_addr.as_mut() }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum PageTableType {
    Sv32 = 2, Sv39 = 3, Sv48 = 4,
}

/// A recursive page table is a last level page table with an entry mapped to the table itself.
///
/// This struct implements the `Mapper` trait.
pub struct RecursivePageTable<'a> {
    root_table: &'a mut PageTable,
    /// Recursive index as `R`
    ///
    /// `R`:   point to root frame, flags: V
    /// `R+1`: point to root frame, flags: V+R+W
    /// `R+2`: point to temp frame, flags: V+R+W
    ///
    /// At any time, we can access root page table through (R, R, ..., R+1, 0).
    /// To access a temp frame, first set `root[R+2]` pointing to the frame with R+W+X,
    /// then we can access the frame through (R, R, ..., R+2, 0).
    rec_idx: usize,
    ///
    temp_map: TempMap<'a>,
    /// Page table type
    #[cfg(riscv64)]
    type_: PageTableType,
}

/// An error indicating that the given page table is not recursively mapped.
///
/// Returned from `RecursivePageTable::new`.
#[derive(Debug)]
pub struct NotRecursivelyMapped;

#[cfg(riscv32)]
impl<'a> RecursivePageTable<'a> {
    /// Creates a new RecursivePageTable from the passed level 2 PageTable.
    ///
    /// The page table must be recursively mapped, that means:
    ///
    /// - The page table must have one recursive entry, i.e. an entry that points to the table
    ///   itself.
    /// - The page table must be active, i.e. the satp register must contain its physical address.
    ///
    /// Otherwise `Err(NotRecursivelyMapped)` is returned.
    pub fn new(table: &'a mut PageTable) -> Result<Self, NotRecursivelyMapped> {
        let page = Page::of_addr(VirtAddr::new(table as *const _ as usize));
        let rec_idx = page.p2_index();

        let satp_frame = crate::register::satp::read().frame();
        if page.p1_index() != rec_idx + 1
            || satp_frame != table[rec_idx].frame()
            || satp_frame != table[rec_idx + 1].frame()
            || !table[rec_idx].flags().contains(F::VALID)
            ||  table[rec_idx].flags().contains(F::READABLE | F::WRITABLE)
            || !table[rec_idx + 1].flags().contains(F::VALID | F::READABLE | F::WRITABLE)
        {
            return Err(NotRecursivelyMapped);
        }

        Ok(RecursivePageTable {
            root_table: table,
            rec_idx,
            temp_map: unsafe { TempMap::new(rec_idx) },
        })
    }

    /// Creates a new RecursivePageTable without performing any checks.
    ///
    /// The `recursive_index` parameter must be the index of the recursively mapped entry.
    pub unsafe fn new_unchecked(table: &'a mut PageTable, recursive_index: usize) -> Self {
        RecursivePageTable {
            root_table: table,
            rec_idx: recursive_index,
            temp_map: TempMap::new(recursive_index),
        }
    }

    fn create_p1_if_not_exist(&mut self, p2_index: usize, allocator: &mut impl FrameAllocator) -> Result<&mut PageTable, MapToError> {
        assert!(p2_index < self.rec_idx || p2_index > self.rec_idx + 2, "invalid p2_index");
        if self.root_table[p2_index].is_unused() {
            let frame = allocator.alloc().ok_or(MapToError::FrameAllocationFailed)?;
            self.root_table[p2_index].set(frame.clone(), F::VALID);
            let p1_table = self.temp_map.map(frame);
            p1_table.zero();
            Ok(p1_table)
        } else {
            let frame = self.root_table[p2_index].frame();
            let p1_table = self.temp_map.map(frame);
            Ok(p1_table)
        }
    }
}

#[cfg(riscv64)]
impl<'a> RecursivePageTable<'a> {
    pub fn new(table: &'a mut PageTable, type_: PageTableType) -> Result<Self, NotRecursivelyMapped> {
        let page = Page::of_addr(VirtAddr::new(table as *const _ as usize));
        let rec_idx = match type_ {
            PageTableType::Sv39 => page.p3_index(),
            PageTableType::Sv48 => page.p4_index(),
            _ => panic!("invalid page table type"),
        };

        use register::satp;
        if page.p3_index() != rec_idx
            || page.p2_index() != rec_idx
            || page.p1_index() != rec_idx + 1
                // Denote recursive_index with l.
                // Require the virtaddr of the root page table to be
                // (p4=l, p3=l, p2=l, p1=l+1, p0=0)
            || satp::read().frame() != table[rec_idx].frame()
            || satp::read().frame() != table[rec_idx + 1].frame()
                // Require that table[l] and table[l+1] maps back to table
            || !table[rec_idx].flags().contains(F::VALID)
            ||  table[rec_idx].flags().contains(F::READABLE | F::WRITABLE)
                // Require that table[l] must be valid, and points to a page table.
            || !table[rec_idx + 1].flags().contains(F::VALID | F::READABLE | F::WRITABLE)
                // Require that table[l+1] must be valid, and points to a page.
        {
            return Err(NotRecursivelyMapped);
        }

        Ok(RecursivePageTable {
            root_table: table,
            rec_idx,
            temp_map: unsafe { TempMap::new(rec_idx, type_) },
            type_,
        })
    }

    pub unsafe fn new_unchecked(table: &'a mut PageTable, recursive_index: usize, type_: PageTableType) -> Self {
        RecursivePageTable {
            root_table: table,
            rec_idx: recursive_index,
            temp_map: TempMap::new(recursive_index, type_),
            type_,
        }
    }

    fn create_p1_if_not_exist(&mut self, page: Page, allocator: &mut impl FrameAllocator)
        -> Result<&mut PageTable, MapToError>
    {
        assert!(page.p4_index() < self.rec_idx || page.p4_index() > self.rec_idx + 2, "invalid p4_index");
        let p4_table = &mut self.root_table;

        let p3_table = if self.type_ == PageTableType::Sv39 {
            assert!(page.p3_index() < self.rec_idx || page.p3_index() > self.rec_idx + 2, "invalid p3_index");
            &mut self.root_table
        } else if p4_table[page.p4_index()].is_unused() {
            let frame = allocator.alloc().ok_or(MapToError::FrameAllocationFailed)?;
            p4_table[page.p4_index()].set(frame, F::VALID);
            // NLL: auto release `&mut p4_table` aka `&mut self.temp_map` here
            let table = self.temp_map.map(frame);
            table.zero();
            table
        } else {
            let frame = p4_table[page.p4_index()].frame();
            self.temp_map.map(frame)
        };

        let p2_table = if p3_table[page.p3_index()].is_unused() {
            let frame = allocator.alloc().ok_or(MapToError::FrameAllocationFailed)?;
            p3_table[page.p3_index()].set(frame, F::VALID);
            let table = self.temp_map.map(frame);
            table.zero();
            table
        } else {
            let frame = p3_table[page.p3_index()].frame();
            self.temp_map.map(frame)
        };

        let p1_table = if p2_table[page.p2_index()].is_unused() {
            let frame = allocator.alloc().ok_or(MapToError::FrameAllocationFailed)?;
            p2_table[page.p2_index()].set(frame, F::VALID);
            let table = self.temp_map.map(frame);
            table.zero();
            table
        } else {
            let frame = p2_table[page.p2_index()].frame();
            self.temp_map.map(frame)
        };

        Ok(p1_table)
    }

    fn ref_p1(&mut self, page: Page) -> Option<&mut PageTable>
    {
        assert!(page.p4_index() < self.rec_idx || page.p4_index() > self.rec_idx + 2, "invalid p4_index");

        let p4_table = &mut self.root_table;

        let p3_table = if self.type_ == PageTableType::Sv39 {
            assert!(page.p3_index() < self.rec_idx || page.p3_index() > self.rec_idx + 2, "invalid p3_index");
            &mut self.root_table
        } else {
            if p4_table[page.p4_index()].is_unused() {
                return None;
            }
            let frame = p4_table[page.p4_index()].frame();
            self.temp_map.map(frame)
        };

        if p3_table[page.p3_index()].is_unused() {
            return None;
        }
        let p2_table = {
            let frame = p3_table[page.p3_index()].frame();
            self.temp_map.map(frame)
        };

        if p2_table[page.p2_index()].is_unused() {
            return None;
        }
        let p1_table = {
            let frame = p2_table[page.p2_index()].frame();
            self.temp_map.map(frame)
        };

        Some(p1_table)
    }
}

#[cfg(riscv32)]
impl<'a> Mapper for RecursivePageTable<'a> {
    fn map_to(&mut self, page: Page, frame: Frame, flags: PageTableFlags, allocator: &mut impl FrameAllocator)
        -> Result<MapperFlush, MapToError>
    {
        let p1_table = self.create_p1_if_not_exist(page.p2_index(), allocator)?;
        if !p1_table[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped);
        }
        p1_table[page.p1_index()].set(frame, flags);
        Ok(MapperFlush::new(page))
    }

    fn unmap(&mut self, page: Page) -> Result<(Frame, MapperFlush), UnmapError> {
        if self.root_table[page.p2_index()].is_unused() {
            return Err(UnmapError::PageNotMapped);
        }
        let p1_frame = self.root_table[page.p2_index()].frame();
        let p1_table = self.temp_map.map(p1_frame);
        let p1_entry = &mut p1_table[page.p1_index()];
        if !p1_entry.flags().contains(F::VALID) {
            return Err(UnmapError::PageNotMapped);
        }
        let frame = p1_entry.frame();
        p1_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    fn ref_entry(&mut self, page: Page) -> Result<&mut PageTableEntry, FlagUpdateError> {
        if self.root_table[page.p2_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        let p1_frame = self.root_table[page.p2_index()].frame();
        let p1_table = self.temp_map.map(p1_frame);
        Ok(&mut p1_table[page.p1_index()])
    }
}

#[cfg(riscv64)]
impl<'a> Mapper for RecursivePageTable<'a> {
    fn map_to(&mut self, page: Page, frame: Frame, flags: PageTableFlags, allocator: &mut impl FrameAllocator)
        -> Result<MapperFlush, MapToError>
    {
        let p1 = self.create_p1_if_not_exist(page, allocator)?;
        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped);
        }
        p1[page.p1_index()].set(frame, flags);
        Ok(MapperFlush::new(page))
    }

    fn unmap(&mut self, page: Page) -> Result<(Frame, MapperFlush), UnmapError> {
        let p1_table = self.ref_p1(page).ok_or(UnmapError::PageNotMapped)?;
        let p1_entry = &mut p1_table[page.p1_index()];
        if !p1_entry.flags().contains(F::VALID) {
            return Err(UnmapError::PageNotMapped);
        }
        let frame = p1_entry.frame();
        p1_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    fn ref_entry(&mut self, page: Page) -> Result<&mut PageTableEntry, FlagUpdateError> {
        let p1_table = self.ref_p1(page).ok_or(FlagUpdateError::PageNotMapped)?;
        Ok(&mut p1_table[page.p1_index()])
    }
}
