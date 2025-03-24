use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::AtomicBool;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct BumpAllocator {
    free_memory_top: AtomicUsize, // Top of free memory
    bump_ptr: AtomicUsize,        // Atomic bump pointer
    lock: AtomicUsize,            // Spinlock for synchronizing memory allocations
}

impl BumpAllocator {
    /// Creates a new bump allocator for a specific memory range
    pub const fn new() -> Self {
        Self {
            free_memory_top: AtomicUsize::new(0),
            bump_ptr: AtomicUsize::new(0),
            lock: AtomicUsize::new(0), // 0 indicates unlocked
        }
    }

    /// Initializes the bump allocator with a given memory range
    /// The allocator will only allocate memory after `init` is called
    pub fn init(&self, free_memory_bottom: usize, free_memory_top: usize) {
        if !ALLOCATOR_INITIALIZED.load(Ordering::SeqCst) {
            self.bump_ptr.store(free_memory_bottom, Ordering::SeqCst);
            self.free_memory_top.store(free_memory_top, Ordering::SeqCst); // i don't actually know if this one needs to be atomic but wtv
            ALLOCATOR_INITIALIZED.store(true, Ordering::SeqCst); // Set the flag to indicate that the allocator is initialized
        } else {
            panic!("Allocator already initialized");
        }
    }

    /// Allocates free memory
    pub fn alloc(&self, layout: Layout) -> *mut u8 {
        if !ALLOCATOR_INITIALIZED.load(Ordering::SeqCst) {
            panic!("Allocator not initialized");
        }

        let size = layout.size();
        let align = layout.align();

        loop {
            // Try to acquire the spinlock (lock == 0 means unlocked)
            if self.lock.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                let current = self.bump_ptr.load(Ordering::Relaxed);

                // Align the allocation address
                let aligned = (current + align - 1) & !(align - 1);
                let next = aligned + size;

                // Check if we have enough space
                if next > self.free_memory_top.load(Ordering::Relaxed) {
                    self.lock.store(0, Ordering::Release); // Release the lock
                    panic!("Out of memory");
                    // Since no deallocation occurs, running out of memory is an unrecoverable error
                }

                // Try to atomically update the bump pointer
                if self.bump_ptr.compare_exchange(current, next, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                    self.lock.store(0, Ordering::Release); // Release the lock
                    return aligned as *mut u8;
                }

                // If allocation failed, release the lock
                self.lock.store(0, Ordering::Release);
            }

            // If we couldn't acquire the lock, keep trying
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // The bump allocator does not deallocate
    }
}

/// Flag for whether allocator is initialized (should only happen once)
static ALLOCATOR_INITIALIZED: AtomicBool = AtomicBool::new(false);

// The tests will not operate without an allocator, so this cfg is necessary
#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: BumpAllocator = BumpAllocator::new();

#[cfg(test)]
mod tests {
    use super::*;
    use core::alloc::Layout;
    extern crate std;

    #[test]
    fn test_basic_alloc() {
        let mut allocator = BumpAllocator::new();
        allocator.init(0x1000, 0x2000);

        let layout = Layout::from_size_align(16, 8).unwrap();
        let ptr = allocator.alloc(layout);
        assert!(!ptr.is_null(), "Allocation failed");
        assert_eq!(ptr as usize, 0x1000, "Unexpected allocation address");
    }

    // other things that would be good to test
    // Box
    // consecutive allocs
    // diff alignments
    // out of memory
    // uninitialized (should panic)
}
