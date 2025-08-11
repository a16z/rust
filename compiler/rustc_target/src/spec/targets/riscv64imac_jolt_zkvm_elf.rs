use crate::spec::{
    Cc, CodeModel, LinkerFlavor, Lld, PanicStrategy, RelocModel, SanitizerSet, Target,
    TargetOptions,
};

pub(crate) fn target() -> Target {
    Target {
        data_layout: "e-m:e-p:64:64-i64:64-i128:128-n32:64-S128".into(),
        llvm_target: "riscv64".into(),
        metadata: crate::spec::TargetMetadata {
            description: Some("Jolt's Zero's zero-knowledge Virtual Machine (RV64IMAC ISA)".into()),
            tier: Some(3),
            host_tools: Some(false),
            std: None,
        },
        pointer_width: 64,
        arch: "riscv64".into(),

        options: TargetOptions {
            os: "zkvm".into(),
            vendor: "jolt".into(),
            linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
            linker: Some("rust-lld".into()),
            cpu: "generic-rv64".into(),
            max_atomic_width: Some(64),
            atomic_cas: true,
            executables: true,
            features: "+m,+a,+c".into(),
            llvm_abiname: "lp64".into(),
            panic_strategy: PanicStrategy::Abort,
            relocation_model: RelocModel::Static,
            code_model: Some(CodeModel::Medium),
            emit_debug_gdb_scripts: false,
            eh_frame_header: false,
            supported_sanitizers: SanitizerSet::KERNELADDRESS | SanitizerSet::SHADOWCALLSTACK,
            singlethread: true,
            supports_stack_protector: false,
            ..Default::default()
        },
    }
}