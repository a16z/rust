use crate::spec::{Cc, LinkerFlavor, Lld, PanicStrategy, RelocModel};
use crate::spec::{Target, TargetOptions};

pub(crate) fn target() -> Target {
    Target {
        data_layout: "e-m:e-p:32:32-i64:64-n32-S128".into(),
        llvm_target: "riscv32".into(),
        pointer_width: 32,
        arch: "riscv32".into(),

        metadata: crate::spec::TargetMetadata {
            description: Some("Jolt's Zero's zero-knowledge Virtual Machine (RV32IM ISA)".into()),
            tier: Some(3),
            host_tools: Some(false),
            std: None,
        },

        options: TargetOptions {
            os: "zkvm".into(),
            vendor: "jolt".into(),
            linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
            linker: Some("rust-lld".into()),
            cpu: "generic-rv32".into(),
            features: "+m".into(),
            max_atomic_width: Some(64),
            atomic_cas: true,
            executables: true,
            llvm_abiname: "ilp32".into(),
            panic_strategy: PanicStrategy::Abort,
            relocation_model: RelocModel::Static,
            emit_debug_gdb_scripts: false,
            eh_frame_header: false,
            singlethread: true,
            supports_stack_protector: false,
            ..Default::default()
        },
    }
}