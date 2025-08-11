use crate::sys::pal::abi::sys_rand;

pub fn fill_bytes(bytes: &mut [u8]) {
    unsafe {
        sys_rand(bytes.as_mut_ptr().cast(), bytes.len());
    }
}
