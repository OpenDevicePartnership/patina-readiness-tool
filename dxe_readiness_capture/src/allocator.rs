use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init(free_memory_bottom: usize, free_memory_top: usize) {
    let heap_start = free_memory_bottom as *mut u8;
    let heap_size = free_memory_top - free_memory_bottom;
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}
