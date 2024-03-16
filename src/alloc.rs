const TOTAL_MEM_SIZE: usize = 64 * 1024;
const NON_HEAP_STATIC_ALLOC_SPACE: usize = 15 * 1024; // 10 KB
const STACK_TOP_ADDR: usize = 14752;

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() {
    let heap_start = STACK_TOP_ADDR + NON_HEAP_STATIC_ALLOC_SPACE;
    let heap_end = TOTAL_MEM_SIZE;
    let heap_size = heap_end - heap_start;
    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
}