use crate::{alloc::{GlobalAlloc, Layout, System}, cell::UnsafeCell};
use super::abi::_HEAP_PTR;

static mut BUMP_ALLOC: BumpAllocator = BumpAllocator::new();

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        BUMP_ALLOC.alloc(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

pub struct BumpAllocator {
    offset: UnsafeCell<usize>,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            offset: UnsafeCell::new(0),
        }
    }

    pub fn free_memory(&self) -> usize {
        heap_start() + (self.offset.get() as usize)
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let alloc_start = align_up(self.free_memory(), layout.align());
        let alloc_end = alloc_start + layout.size();
        *self.offset.get() = alloc_end - self.free_memory();

        alloc_start as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

unsafe impl Sync for BumpAllocator {}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

fn heap_start() -> usize {
    unsafe { _HEAP_PTR as *const u8 as usize }
}
