
extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};

struct HeaplessGlobalAllocator<const N: usize> {
    buffer: heapless::Vec<u8, N>,
}

unsafe impl<const N: usize> GlobalAlloc for HeaplessGlobalAllocator<N> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() <= N {
            0x100 as *mut u8
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Heapless doesn't support deallocation, so we do nothing here
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: HeaplessGlobalAllocator<40000> = HeaplessGlobalAllocator {
    buffer: heapless::Vec::new(),
};