//! Traits for abstracting away frame allocation and deallocation.

use addr::Frame;

/// A trait for types that can allocate a frame of memory.
pub trait FrameAllocator {
    /// Allocate a frame of the appropriate size and return it if possible.
    fn alloc(&mut self) -> Option<Frame>;
}

/// A trait for types that can deallocate a frame of memory.
pub trait FrameDeallocator {
    /// Deallocate the given frame of memory.
    fn dealloc(&mut self, frame: Frame);
}
