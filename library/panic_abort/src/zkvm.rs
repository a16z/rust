use core::panic::PanicPayload;
use alloc::string::String;

extern "C" {
    fn sys_panic(msg_ptr: *const u8, len: usize) -> !;
}

pub(crate) unsafe fn zkvm_set_abort_message(payload: &mut dyn PanicPayload) {
    let payload = payload.get();
    let msg = match payload.downcast_ref::<&'static str>() {
        Some(msg) => msg.as_bytes(),
        None => match payload.downcast_ref::<String>() {
            Some(msg) => msg.as_bytes(),
            None => &[],
        },
    };

    sys_panic(msg.as_ptr(), msg.len())
}
