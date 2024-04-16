// use alloc::string::String;
use core::panic::PanicPayload;

pub(crate) unsafe fn zkvm_set_abort_message(_payload: &mut dyn PanicPayload) {
    // TODO: fix

    // let payload = payload.get();
    // let msg = match payload.downcast_ref::<&'static str>() {
    //     Some(msg) => msg.as_bytes(),
    //     None => match payload.downcast_ref::<String>() {
    //         Some(msg) => msg.as_bytes(),
    //         None => &[],
    //     },
    // };
    // if msg.is_empty() {
    //     return;
    // }

    // extern "C" {
    //     fn sys_panic(msg_ptr: *const u8, len: usize) -> !;
    // }

    // sys_panic(msg.as_ptr(), msg.len());
}
