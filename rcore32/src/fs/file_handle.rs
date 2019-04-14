use alloc::{string::String, sync::Arc};

use rcore_fs::vfs::{FsError, INode, Metadata, Result};

#[derive(Clone)]
pub struct FileHandle {
    inode: Arc<INode>,  // 指向文件对应的INode的指针
    offset: u64,    // 当前handle在文件中的位置
}

impl FileHandle {
    pub fn new(inode : Arc<INode> ) -> Self {
        FileHandle {
            inode,
            offset: 0,
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        let len = self.read_at(self.offset as usize, buf).unwrap();
        self.offset += len as u64;
        Some(len)
    }

    pub fn read_at(&mut self, offset: usize, buf: &mut [u8]) -> Option<usize> {
        if let Ok(len) = self.inode.read_at(offset, buf) {
            return Some(len);
        }
        None
    }

}
