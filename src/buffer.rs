//! Endpoint memory buffers
//!
//! A `USB` instance owns an `Allocator`. The `Allocator` hands-off
//! `Buffer`s from a single, large byte collection. `Buffer`s support
//! bulk, volatile reads and writes.

/// Endpoint memory buffer allocator
pub struct Allocator {
    start: *mut u8,
    ptr: *mut u8,
}

// Safety: OK to send across execution contexts, because
// the wrapped memory is static.
unsafe impl Send for crate::buffer::Allocator {}

impl Allocator {
    /// Create a memory allocator that allocates block from static, mutable memory.
    pub fn new(buffer: &'static mut [u8]) -> Self {
        let start = buffer.as_mut_ptr();
        let ptr = unsafe { start.add(buffer.len()) };
        Allocator { start, ptr }
    }

    /// Allocates a buffer of `size`
    ///
    /// The pointer returned from `allocate` is guaranteed to be at least `size`
    /// bytes large.
    pub fn allocate(&mut self, size: usize) -> Option<Buffer> {
        let ptr = self.ptr as usize;
        let ptr = ptr.checked_sub(size)?;
        let start = self.start as usize;
        if ptr < start {
            None
        } else {
            self.ptr = ptr as *mut u8;
            Some(Buffer {
                ptr: self.ptr,
                len: size,
            })
        }
    }

    /// Represents an `Allocator` that does not allocate any memory
    pub fn empty() -> Self {
        Allocator {
            start: core::ptr::null_mut(),
            ptr: core::ptr::null_mut(),
        }
    }
}

/// An endpoint memory buffer that derives from static memory                                                                                                                                    
pub struct Buffer {
    ptr: *mut u8,
    len: usize,
}

// Safety: OK to send `Buffer` across execution contexts. It's
// assumed to point to static memory.
unsafe impl Send for Buffer {}

impl Buffer {
    /// Read the contents of this buffer into `buffer`, returning
    /// how many elements were read
    ///
    /// All reads from this buffer are volatile.
    pub fn volatile_read(&self, buffer: &mut [u8]) -> usize {
        let size = buffer.len().min(self.len);
        buffer
            .iter_mut()
            .take(size)
            .fold(self.ptr, |src, dst| unsafe {
                // Safety: pointer valid for `len` elements, the take() prevents
                // us from going out of bounds.
                *dst = src.read_volatile();
                src.add(1)
            });
        size
    }

    /// Write the contents from `buffer` into this memory buffer,
    /// returning how many elements were written
    ///
    /// All writes into this buffer are volatile.
    pub fn volatile_write(&mut self, buffer: &[u8]) -> usize {
        let size = buffer.len().min(self.len);
        buffer.iter().take(size).fold(self.ptr, |dst, src| unsafe {
            // Safety: pointer valid for `len` elements, the take() prevents
            // us from going out of bounds.
            dst.write_volatile(*src);
            dst.add(1)
        });
        size
    }

    /// Returns the start of this memory buffer
    pub fn as_ptr_mut(&mut self) -> *mut u8 {
        self.ptr
    }
}

#[cfg(test)]
mod test {
    use super::Allocator;

    #[test]
    fn allocate_entire_buffer() {
        static mut BUFFER: [u8; 32] = [0; 32];
        let mut alloc = unsafe { Allocator::new(&mut BUFFER) };
        let ptr = alloc.allocate(32);
        assert!(ptr.is_some());
        assert_eq!(ptr.unwrap().ptr, unsafe { BUFFER.as_mut_ptr() });

        let ptr = alloc.allocate(1);
        assert!(ptr.is_none());
    }

    #[test]
    fn allocate_partial_buffers() {
        static mut BUFFER: [u8; 32] = [0; 32];
        let mut alloc = unsafe { Allocator::new(&mut BUFFER) };

        let ptr = alloc.allocate(7);
        assert!(ptr.is_some());
        assert_eq!(ptr.unwrap().ptr, unsafe { BUFFER.as_mut_ptr().add(32 - 7) });

        let ptr = alloc.allocate(7);
        assert!(ptr.is_some());
        assert_eq!(ptr.unwrap().ptr, unsafe {
            BUFFER.as_mut_ptr().add(32 - 14)
        });

        let ptr = alloc.allocate(19);
        assert!(ptr.is_none());
    }

    #[test]
    fn allocate_empty() {
        let mut alloc = Allocator::empty();
        assert!(alloc.allocate(1).is_none());
    }
}
