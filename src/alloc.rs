use buddy_alloc::{BuddyAllocParam, FastAllocParam, NonThreadsafeAlloc};

// These values can be tuned
const NON_HEAP_STATIC_ALLOC_SPACE: usize = 15 * 1024; // 10 KB
const FAST_HEAP_SIZE: usize = 1 * 1024; // 1 KB
const HEAP_SIZE: usize = 30 * 1024; // 32 KB
const LEAF_SIZE: usize = 64;

// Stack pointer starts here, and decreases. Therefore
// right after this address is static space.
const STACK_TOP_ADDR: usize = 14752;

// The fast heap is right after our static memory.
const FAST_HEAP_ADDR: usize = STACK_TOP_ADDR + NON_HEAP_STATIC_ALLOC_SPACE;

// The slow heap is right after the fast heap.
const HEAP_ADDR: usize = FAST_HEAP_ADDR + FAST_HEAP_SIZE;

#[global_allocator]
static ALLOC: NonThreadsafeAlloc = {
    let fast_param = FastAllocParam::new(FAST_HEAP_ADDR as *const u8, FAST_HEAP_SIZE);
    let buddy_param = BuddyAllocParam::new(HEAP_ADDR as *const u8, HEAP_SIZE, LEAF_SIZE);
    NonThreadsafeAlloc::new(fast_param, buddy_param)
};
