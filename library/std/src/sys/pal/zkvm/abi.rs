unsafe extern "C" {
    pub fn sys_rand(dest: *mut u8, words: usize);
    pub fn sys_alloc(size: usize, align: usize) -> *mut u8;
}