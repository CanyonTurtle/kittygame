use buddy_alloc::{BuddyAllocParam, FastAllocParam, NonThreadsafeAlloc};

// These values can be tuned
const FAST_HEAP_SIZE: usize = 1 * 1024; // 1 KB
const HEAP_SIZE: usize = 32 * 1024; // 32 KB
const LEAF_SIZE: usize = 64;

const FAST_HEAP_ADDR: usize = 24000;
const HEAP_ADDR: usize = FAST_HEAP_ADDR + FAST_HEAP_SIZE;
// static mut FAST_HEAP: [u8; FAST_HEAP_SIZE] = [0u8; FAST_HEAP_SIZE];
// static mut HEAP: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];

#[global_allocator]
static ALLOC: NonThreadsafeAlloc = {
    let fast_param = FastAllocParam::new(FAST_HEAP_ADDR as *const u8, FAST_HEAP_SIZE);
    let buddy_param = BuddyAllocParam::new(HEAP_ADDR as *const u8, HEAP_SIZE, LEAF_SIZE);
    NonThreadsafeAlloc::new(fast_param, buddy_param)
};
