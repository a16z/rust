use super::abi::sys_alloc;
use crate::alloc::{GlobalAlloc, Layout, System};

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { sys_alloc(layout.size(), layout.align()) }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
