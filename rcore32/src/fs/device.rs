use spin::RwLock;
use core::mem::uninitialized;
use rcore_fs::dev::*;

pub struct MemBuf(RwLock<&'static mut [u8]>);

impl MemBuf {
    pub unsafe fn new(begin: unsafe extern "C" fn(), end: unsafe extern "C" fn()) -> Self {
        use core::slice;
        MemBuf(RwLock::new(slice::from_raw_parts_mut(
            begin as *mut u8,
            end as usize - begin as usize,
        )))
    }
}

impl Device for MemBuf {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Option<usize> {
        let slice = self.0.read();
        let len = buf.len().min(slice.len() - offset);  // 取磁盘剩余长度和ｂｕｆ大小的较小值
        buf[..len].copy_from_slice(&slice[offset..offset + len]);
        Some(len)
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> Option<usize> {
        let mut slice = self.0.write();
        let len = buf.len().min(slice.len() - offset);
        slice[offset..offset + len].copy_from_slice(&buf[..len]);
        Some(len)
    }

}

//impl Device{
    //fn read_block(&self, id: usize, offset: usize, buf: &mut [u8]) -> bool {
        //match self.read_at(id * BLKSIZE + offset, buf) {
            //Some(len) if len == buf.len() => {return true;},
            //_ => {return false;},
        //}
    //}

    //pub fn load_struct<T: AsBuf>(&self, id: usize) -> Option<T> {
        //let mut s: T = unsafe { uninitialized() };
        //if self.read_block(id, 0, s.as_buf_mut()) {
            //return Some(s);
        //}
        //None
    //}
//}
