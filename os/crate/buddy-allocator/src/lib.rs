
#![feature(alloc)]
#![no_std]
#![feature(lang_items)]

extern crate alloc;
use alloc::vec::Vec;
use core::alloc::Layout;

pub struct BuddyAllocator {
    nodes : Vec<i8>,
    level : u8,
}

impl BuddyAllocator {
    pub fn new() -> Self {
        let ret = BuddyAllocator{
            nodes : Vec::new(),
            level : 0,
        };
        ret
    }

    pub fn init(&mut self, level : u8) {
        self.level = level;
        let mut idx = 0;
        for s in 0..level {
            for _t in 0..(1 << s) {
                self.nodes.push(0_i8);
                self.nodes[idx] = ( level - s - 1 ) as i8;
                idx = idx + 1;
            }
        }
    }

    pub fn alloc(&mut self, alloc_size : usize) -> Option<usize> {
        let size = log2_up(alloc_size) as i8;
        let mut location = 0;
        let mut height = self.level - 1;
        let ret;
        while height != 0 {
            if size > self.nodes[location] {
                panic!("memory is not enough ");
            }
            if self.nodes[(location << 1) + 1] >= size {
                location = (location << 1) + 1;
            }else if self.nodes[(location + 1) << 1] >= size{
                location = (location + 1) << 1;
            }else{
                break;
            }
            height = height - 1;
        }
        ret = (location + 1) * (1 << height) - (1 << (self.level - 1));
        self.nodes[location] = -1;
        while location > 0 {    // 回溯，更新父辈节点
            if location & 0x1 > 0 { // 当前节点的下标为奇数
                if self.nodes[location] > self.nodes[location + 1] {    // 当前节点的值大于兄弟节点的值
                    self.nodes[location >> 1] = self.nodes[location];
                }else{ // 兄弟节点的值不小于当前节点的值
                    self.nodes[location >> 1] = self.nodes[location + 1];
                }
                location = location >> 1;
            }else{  //当前节点的下标为偶数
                if self.nodes[location] > self.nodes[location - 1] {
                    self.nodes[(location - 1) >> 1] = self.nodes[location];
                }else{
                    self.nodes[(location - 1) >> 1] = self.nodes[location - 1];
                }
                location = (location - 1) >> 1;
            }
        }
        Some(ret)
    }

    pub fn dealloc(&mut self, address : usize, dealloc_size : usize){
        let size = log2_down(dealloc_size) as i8;
        let mut location = address + (1 << (self.level - 1)) - 1;
        let mut height = 0_i8;
        while size > height {
            height += 1;
            location = if location & 0x1 == 0 {
                (location - 1) >> 1
            }else{
                location >> 1
            };
        }
        self.nodes[location] = size as i8;
        while location > 0 {
            if location & 0x1 > 0 { //　奇数下标
                if self.nodes[location] == self.nodes[location + 1] && self.nodes[location] == height{
                    self.nodes[location >> 1] = self.nodes[location] + 1;
                }else if self.nodes[location] > self.nodes[location >> 1] {
                    self.nodes[location >> 1] = self.nodes[location];
                }
                location = location >> 1;
            }else{ // 偶数下标
                if self.nodes[location] == self.nodes[location - 1] && self.nodes[location] == height{
                    self.nodes[(location - 1) >> 1] = self.nodes[location] + 1;
                }else if self.nodes[location] > self.nodes[(location - 1) >> 1]{
                    self.nodes[(location - 1) >> 1] = self.nodes[location];
                }
                location = (location - 1) >> 1;
            }
            height = height + 1;
        }
    }
}


#[inline(always)]
fn log2_up(x: usize) -> usize {    // 以２为底的对数向上取整的值，主要考虑分配内存时应该向上取整
    assert_ne!(x, 0);
    let mut temp_x = x;
    let mut pos = -1;
    while temp_x != 0 {
        pos += 1;
        temp_x >>= 1;
    }
    if x - (1 << pos) != 0 {
        pos = pos + 1;
    }
    pos as usize
}

#[inline(always)]
pub fn log2_down(x: usize) -> usize {    // 以２为底的对数向下取整的值,释放内存时向下取整
    assert_ne!(x, 0);
    let mut temp_x = x;
    let mut pos = -1;
    while temp_x != 0 {
        pos += 1;
        temp_x >>= 1;
    }
    pos as usize
}

#[lang = "oom"]
fn oom(_: Layout) -> ! {
    panic!("out of memory");
} 