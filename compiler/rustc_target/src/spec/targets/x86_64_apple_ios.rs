use crate::spec::base::apple::{ios_sim_llvm_target, opts, Arch, TargetAbi};
use crate::spec::{SanitizerSet, Target, TargetOptions};

pub fn target() -> Target {
    let arch = Arch::X86_64;
    // x86_64-apple-ios is a simulator target, even though it isn't declared
    // that way in the target name like the other ones...
    let mut base = opts("ios", arch, TargetAbi::Simulator);
    base.supported_sanitizers = SanitizerSet::ADDRESS | SanitizerSet::THREAD;

    Target {
        llvm_target: ios_sim_llvm_target(arch).into(),
        metadata: crate::spec::TargetMetadata {
            description: Some("64-bit x86 iOS".into()),
            tier: Some(2),
            host_tools: Some(false),
            std: Some(true),
        },
        pointer_width: 64,
        data_layout:
            "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128".into(),
        arch: arch.target_arch(),
        options: TargetOptions { max_atomic_width: Some(128), ..base },
    }
}
