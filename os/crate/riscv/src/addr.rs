use bit_field::BitField;


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(usize);


impl VirtAddr {
    #[cfg(riscv32)]
    pub fn new(addr: usize) -> VirtAddr {
        VirtAddr(addr)
    }

    #[cfg(riscv64)]
    pub fn new(addr: usize) -> VirtAddr {
        if addr.get_bit(47) {
            assert!(addr.get_bits(48..64) == 0xFFFF, "va 48..64 is not sext");
        } else {
            assert!(addr.get_bits(48..64) == 0x0000, "va 48..64 is not sext");
        }
        VirtAddr(addr)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }

    #[cfg(riscv64)]
    pub fn p4_index(&self) -> usize {
        self.0.get_bits(39..48)
    }

    #[cfg(riscv64)]
    pub fn p3_index(&self) -> usize {
        self.0.get_bits(30..39)
    }

    #[cfg(riscv64)]
    pub fn p2_index(&self) -> usize {
        self.0.get_bits(21..30)
    }
    #[cfg(riscv32)]
    pub fn p2_index(&self) -> usize {
        self.0.get_bits(22..32)
    }

    #[cfg(riscv64)]
    pub fn p1_index(&self) -> usize {
        return self.0.get_bits(12..21)
    }
    #[cfg(riscv32)]
    pub fn p1_index(&self) -> usize {
        return self.0.get_bits(12..22)
    }

    #[cfg(riscv64)]
    pub fn page_number(&self) -> usize {
        self.0.get_bits(12..64)
    }
    #[cfg(riscv32)]
    pub fn page_number(&self) -> usize {
        self.0.get_bits(12..32)
    }

    pub fn page_offset(&self) -> usize {
        self.0.get_bits(0..12)
    }

    pub fn to_4k_aligned(&self) -> Self {
        VirtAddr((self.0 >> 12) << 12)
    }

    #[cfg(riscv32)]
    pub fn from_page_table_indices(p2_index: usize,
                                   p1_index: usize,
                                   offset: usize) -> Self
    {
        assert!(p2_index.get_bits(10..32) == 0, "p2_index exceeding 10 bits");
        assert!(p1_index.get_bits(10..32) == 0, "p1_index exceeding 10 bits");
        assert!(offset.get_bits(12..32) == 0, "offset exceeding 12 bits");
        VirtAddr::new((p2_index << 22) | (p1_index << 12) | offset)
    }

    #[cfg(riscv64)]
    pub fn from_page_table_indices(p4_index: usize,
                                   p3_index: usize,
                                   p2_index: usize,
                                   p1_index: usize,
                                   offset: usize) -> Self
    {
        assert!(p4_index.get_bits(10..32) == 0, "p4_index exceeding 9 bits");
        assert!(p3_index.get_bits(10..32) == 0, "p3_index exceeding 9 bits");
        assert!(p2_index.get_bits(10..32) == 0, "p2_index exceeding 9 bits");
        assert!(p1_index.get_bits(10..32) == 0, "p1_index exceeding 9 bits");
        assert!(offset.get_bits(12..32) == 0, "offset exceeding 12 bits");
        let mut addr: usize =
            (p4_index << 12 << 9 << 9 << 9) |
            (p3_index << 12 << 9 << 9) |
            (p2_index << 12 << 9) |
            (p1_index << 12) |
            offset;
        if addr.get_bit(47) {
            addr.set_bits(48..64, 0xFFFF);
        } else {
            addr.set_bits(48..64, 0x0000);
        }
        VirtAddr::new(addr)
    }

    pub(crate) unsafe fn as_mut<'a, 'b, T>(&'a self) -> &'b mut T {
        &mut *(self.0 as *mut T)
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(usize);

impl PhysAddr {
    #[cfg(riscv32)]
    pub fn new(addr: usize) -> PhysAddr {
        PhysAddr(addr)
    }

    #[cfg(riscv64)]
    pub fn new(addr: usize) -> PhysAddr {
        assert!(addr.get_bits(32..64) == 0, "pa 32..64 not zero?");
        PhysAddr(addr)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }

    #[cfg(riscv64)]
    pub fn p4_index(&self) -> usize {
        self.0.get_bits(39..48) as usize
    }

    #[cfg(riscv64)]
    pub fn p3_index(&self) -> usize {
        self.0.get_bits(30..39) as usize
    }

    #[cfg(riscv64)]
    pub fn p2_index(&self) -> usize {
        self.0.get_bits(21..30) as usize
    }

    #[cfg(riscv32)]
    pub fn p2_index(&self) -> usize {
        self.0.get_bits(22..32)
    }

    #[cfg(riscv64)]
    pub fn p1_index(&self) -> usize {
        self.0.get_bits(12..21) as usize
    }
    #[cfg(riscv32)]
    pub fn p1_index(&self) -> usize {
        self.0.get_bits(12..22)
    }

    #[cfg(riscv64)]
    pub fn page_number(&self) -> usize {
        self.0.get_bits(12..64) as usize
    }
    #[cfg(riscv32)]
    pub fn page_number(&self) -> usize {
        self.0.get_bits(12..32)
    }

    pub fn page_offset(&self) -> usize {
        self.0.get_bits(0..12)
    }

    pub fn to_4k_aligned(&self) -> Self {
        PhysAddr((self.0 >> 12) << 12)
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page(VirtAddr);


impl Page {
    pub fn of_addr(addr: VirtAddr) -> Self {
        Page(addr.to_4k_aligned())
    }

    pub fn of_vpn(vpn: usize) -> Self {
        Page(VirtAddr::new(vpn << 12))
    }

    pub fn start_address(&self) -> VirtAddr { self.0.clone() }

    #[cfg(riscv64)]
    pub fn p4_index(&self) -> usize { self.0.p4_index() }

    #[cfg(riscv64)]
    pub fn p3_index(&self) -> usize { self.0.p3_index() }

    pub fn p2_index(&self) -> usize { self.0.p2_index() }

    pub fn p1_index(&self) -> usize { self.0.p1_index() }

    pub fn number(&self) -> usize { self.0.page_number() }

    #[cfg(riscv64)]
    pub fn from_page_table_indices(p4_index: usize, p3_index: usize,
                                   p2_index: usize, p1_index: usize) -> Self {
        let mut addr: usize = 0;
        addr.set_bits(39..48, p4_index);
        addr.set_bits(30..39, p3_index);
        addr.set_bits(21..30, p2_index);
        addr.set_bits(12..21, p1_index);
        if addr.get_bit(47) {
            addr.set_bits(48..64, 0xFFFF);
        } else {
            addr.set_bits(48..64, 0x0000);
        }
        Page::of_addr(VirtAddr::new(addr))
    }

    #[cfg(riscv32)]
    pub fn from_page_table_indices(p2_index: usize, p1_index: usize) -> Self {
        let mut addr: usize = 0;
        addr.set_bits(22..32, p2_index);
        addr.set_bits(12..22, p1_index);
        Page::of_addr(VirtAddr::new(addr))
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame(PhysAddr);


impl Frame {
    pub fn of_addr(addr: PhysAddr) -> Self {
        Frame(addr.to_4k_aligned())
    }

    #[inline(always)]
    pub fn of_ppn(ppn: usize) -> Self {
        Frame(PhysAddr::new(ppn << 12))
    }

    pub fn start_address(&self) -> PhysAddr { self.0.clone() }

    #[cfg(riscv64)]
    pub fn p4_index(&self) -> usize { self.0.p4_index() }

    #[cfg(riscv64)]
    pub fn p3_index(&self) -> usize { self.0.p3_index() }

    pub fn p2_index(&self) -> usize { self.0.p2_index() }

    pub fn p1_index(&self) -> usize { self.0.p1_index() }

    pub fn number(&self) -> usize { self.0.page_number() }
}
