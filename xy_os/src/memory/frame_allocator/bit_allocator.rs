extern crate bit_field;
use core::ops::Range;
use bit_field::BitField;

pub trait BitAlloc: Default {
    const CAP: usize; // 共有多少个bit，编号从0～CAP-1
    fn alloc(&mut self) -> Option<usize>; // 分配一个空余bit,对应一个物理页帧
    fn dealloc(&mut self, key: usize); // 回收一个被占用的bit，对应回收一个物理页
    fn insert(&mut self, range: Range<usize>); // 划定一块区域，声明其被占用
    fn remove(&mut self, range: Range<usize>); // 回收一块被占用的区域
    fn any(&self) -> bool; // 是否有空闲bit，即对应是否有空闲物理页
    fn test(&self, key: usize) -> bool; // 检验某一物理页帧是否空闲
}

// Implement the bit allocator by segment tree algorithm.
#[derive(Default)]
pub struct BitAllocCascade16<T: BitAlloc> {
    bitset: u16, // 对每一个bit, 1代表空闲, 0代表被占用
    sub: [T; 16],
}

#[derive(Default)]
pub struct BitAlloc16(u16);
pub type BitAlloc256 = BitAllocCascade16<BitAlloc16>;
pub type BitAlloc4K = BitAllocCascade16<BitAlloc256>;

impl<T: BitAlloc> BitAlloc for BitAllocCascade16<T> {
    const CAP: usize = T::CAP * 16;

    fn alloc(&mut self) -> Option<usize> {
        if self.any() {
            let i = log2(self.bitset);
            let res = self.sub[i].alloc().unwrap() + i * T::CAP;
            self.bitset.set_bit(i, self.sub[i].any());
            Some(res)
        } else {
            None
        }
    }
    fn dealloc(&mut self, key: usize) {
        let i = key / T::CAP;
        self.sub[i].dealloc(key % T::CAP);
        self.bitset.set_bit(i, true);
    }
    fn insert(&mut self, range: Range<usize>) {
        self.for_range(range, |sub: &mut T, range| sub.insert(range));
    }
    fn remove(&mut self, range: Range<usize>) {
        self.for_range(range, |sub: &mut T, range| sub.remove(range));
    }
    fn any(&self) -> bool {
        self.bitset != 0
    }
    fn test(&self, key: usize) -> bool {
        self.sub[key / T::CAP].test(key % T::CAP)
    }
}

impl<T: BitAlloc> BitAllocCascade16<T> {
    fn for_range(&mut self, range: Range<usize>, f: impl Fn(&mut T, Range<usize>)) {
        let Range { start, end } = range;
        assert!(start <= end);
        assert!(end <= Self::CAP);
        for i in start / T::CAP..=(end - 1) / T::CAP {
            let begin = if start / T::CAP == i {
                start % T::CAP
            } else {
                0
            };
            let end = if end / T::CAP == i {
                end % T::CAP
            } else {
                T::CAP
            };
            f(&mut self.sub[i], begin..end);
            self.bitset.set_bit(i, self.sub[i].any());
        }
    }
}

// BitAlloc16 作为线段树的叶子节点
impl BitAlloc for BitAlloc16 {
    const CAP: usize = 16;

    fn alloc(&mut self) -> Option<usize> {
        if self.any() {
            let i = log2(self.0);
            self.0.set_bit(i, false);
            Some(i)
        } else {
            None
        }
    }
    fn dealloc(&mut self, key: usize) {
        assert!(!self.test(key));
        self.0.set_bit(key, true);
    }
    fn insert(&mut self, range: Range<usize>) {
        self.0.set_bits(range.clone(), 0xffff.get_bits(range));
    }
    fn remove(&mut self, range: Range<usize>) {
        self.0.set_bits(range, 0);
    }
    fn any(&self) -> bool {
        self.0 != 0
    }
    fn test(&self, key: usize) -> bool {
        self.0.get_bit(key)
    }
}

#[inline(always)]
fn log2(mut x: u16) -> usize {
    // a naive implement
    assert_ne!(x, 0);
    let mut pos = -1;
    while x != 0 {
        pos += 1;
        x >>= 1;
    }
    pos as usize
}