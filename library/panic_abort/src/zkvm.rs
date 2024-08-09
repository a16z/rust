use core::panic::PanicPayload;

pub(crate) unsafe fn zkvm_set_abort_message(_payload: &mut dyn PanicPayload) {
    extern "C" {
        fn jolt_panic() -> !;
    }

    jolt_panic()
}
