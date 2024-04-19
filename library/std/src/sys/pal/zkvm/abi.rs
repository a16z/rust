extern "C" {
    pub fn sys_alloc(size: usize, align: usize) -> *mut u8;
}
