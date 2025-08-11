use core::panic::PanicPayload;

pub(crate) unsafe fn zkvm_set_abort_message(_payload: &mut dyn PanicPayload) {
    unsafe extern "C" {
        fn jolt_panic() -> !;
    }

    unsafe {
        jolt_panic();
    }
}