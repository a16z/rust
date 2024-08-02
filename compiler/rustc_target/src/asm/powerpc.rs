use std::fmt;

use rustc_span::Symbol;

use super::{InlineAsmArch, InlineAsmType, ModifierInfo};

def_reg_class! {
    PowerPC PowerPCInlineAsmRegClass {
        reg,
        reg_nonzero,
        freg,
        cr,
        xer,
    }
}

impl PowerPCInlineAsmRegClass {
    pub fn valid_modifiers(self, _arch: super::InlineAsmArch) -> &'static [char] {
        &[]
    }

    pub fn suggest_class(self, _arch: InlineAsmArch, _ty: InlineAsmType) -> Option<Self> {
        None
    }

    pub fn suggest_modifier(
        self,
        _arch: InlineAsmArch,
        _ty: InlineAsmType,
    ) -> Option<ModifierInfo> {
        None
    }

    pub fn default_modifier(self, _arch: InlineAsmArch) -> Option<ModifierInfo> {
        None
    }

    pub fn supported_types(
        self,
        arch: InlineAsmArch,
    ) -> &'static [(InlineAsmType, Option<Symbol>)] {
        match self {
            Self::reg | Self::reg_nonzero => {
                if arch == InlineAsmArch::PowerPC {
                    types! { _: I8, I16, I32; }
                } else {
                    types! { _: I8, I16, I32, I64; }
                }
            }
            Self::freg => types! { _: F32, F64; },
            Self::cr | Self::xer => &[],
        }
    }
}

def_regs! {
    PowerPC PowerPCInlineAsmReg PowerPCInlineAsmRegClass {
        r0: reg = ["r0", "0"],
        r3: reg, reg_nonzero = ["r3", "3"],
        r4: reg, reg_nonzero = ["r4", "4"],
        r5: reg, reg_nonzero = ["r5", "5"],
        r6: reg, reg_nonzero = ["r6", "6"],
        r7: reg, reg_nonzero = ["r7", "7"],
        r8: reg, reg_nonzero = ["r8", "8"],
        r9: reg, reg_nonzero = ["r9", "9"],
        r10: reg, reg_nonzero = ["r10", "10"],
        r11: reg, reg_nonzero = ["r11", "11"],
        r12: reg, reg_nonzero = ["r12", "12"],
        r14: reg, reg_nonzero = ["r14", "14"],
        r15: reg, reg_nonzero = ["r15", "15"],
        r16: reg, reg_nonzero = ["r16", "16"],
        r17: reg, reg_nonzero = ["r17", "17"],
        r18: reg, reg_nonzero = ["r18", "18"],
        r19: reg, reg_nonzero = ["r19", "19"],
        r20: reg, reg_nonzero = ["r20", "20"],
        r21: reg, reg_nonzero = ["r21", "21"],
        r22: reg, reg_nonzero = ["r22", "22"],
        r23: reg, reg_nonzero = ["r23", "23"],
        r24: reg, reg_nonzero = ["r24", "24"],
        r25: reg, reg_nonzero = ["r25", "25"],
        r26: reg, reg_nonzero = ["r26", "26"],
        r27: reg, reg_nonzero = ["r27", "27"],
        r28: reg, reg_nonzero = ["r28", "28"],
        f0: freg = ["f0", "fr0"],
        f1: freg = ["f1", "fr1"],
        f2: freg = ["f2", "fr2"],
        f3: freg = ["f3", "fr3"],
        f4: freg = ["f4", "fr4"],
        f5: freg = ["f5", "fr5"],
        f6: freg = ["f6", "fr6"],
        f7: freg = ["f7", "fr7"],
        f8: freg = ["f8", "fr8"],
        f9: freg = ["f9", "fr9"],
        f10: freg = ["f10", "fr10"],
        f11: freg = ["f11", "fr11"],
        f12: freg = ["f12", "fr12"],
        f13: freg = ["f13", "fr13"],
        f14: freg = ["f14", "fr14"],
        f15: freg = ["f15", "fr15"],
        f16: freg = ["f16", "fr16"],
        f17: freg = ["f17", "fr17"],
        f18: freg = ["f18", "fr18"],
        f19: freg = ["f19", "fr19"],
        f20: freg = ["f20", "fr20"],
        f21: freg = ["f21", "fr21"],
        f22: freg = ["f22", "fr22"],
        f23: freg = ["f23", "fr23"],
        f24: freg = ["f24", "fr24"],
        f25: freg = ["f25", "fr25"],
        f26: freg = ["f26", "fr26"],
        f27: freg = ["f27", "fr27"],
        f28: freg = ["f28", "fr28"],
        f29: freg = ["f29", "fr29"],
        f30: freg = ["f30", "fr30"],
        f31: freg = ["f31", "fr31"],
        cr: cr = ["cr"],
        cr0: cr = ["cr0"],
        cr1: cr = ["cr1"],
        cr2: cr = ["cr2"],
        cr3: cr = ["cr3"],
        cr4: cr = ["cr4"],
        cr5: cr = ["cr5"],
        cr6: cr = ["cr6"],
        cr7: cr = ["cr7"],
        xer: xer = ["xer"],
        #error = ["r1", "1", "sp"] =>
            "the stack pointer cannot be used as an operand for inline asm",
        #error = ["r2", "2"] =>
            "r2 is a system reserved register and cannot be used as an operand for inline asm",
        #error = ["r13", "13"] =>
            "r13 is a system reserved register and cannot be used as an operand for inline asm",
        #error = ["r29", "29"] =>
            "r29 is used internally by LLVM and cannot be used as an operand for inline asm",
        #error = ["r30", "30"] =>
            "r30 is used internally by LLVM and cannot be used as an operand for inline asm",
        #error = ["r31", "31", "fp"] =>
            "the frame pointer cannot be used as an operand for inline asm",
        #error = ["lr"] =>
            "the link register cannot be used as an operand for inline asm",
        #error = ["ctr"] =>
            "the counter register cannot be used as an operand for inline asm",
        #error = ["vrsave"] =>
            "the vrsave register cannot be used as an operand for inline asm",
    }
}

impl PowerPCInlineAsmReg {
    pub fn emit(
        self,
        out: &mut dyn fmt::Write,
        _arch: InlineAsmArch,
        _modifier: Option<char>,
    ) -> fmt::Result {
        macro_rules! do_emit {
            (
                $($(($reg:ident, $value:literal)),*;)*
            ) => {
                out.write_str(match self {
                    $($(Self::$reg => $value,)*)*
                })
            };
        }
        // Strip off the leading prefix.
        do_emit! {
            (r0, "0"), (r3, "3"), (r4, "4"), (r5, "5"), (r6, "6"), (r7, "7");
            (r8, "8"), (r9, "9"), (r10, "10"), (r11, "11"), (r12, "12"), (r14, "14"), (r15, "15");
            (r16, "16"), (r17, "17"), (r18, "18"), (r19, "19"), (r20, "20"), (r21, "21"), (r22, "22"), (r23, "23");
            (r24, "24"), (r25, "25"), (r26, "26"), (r27, "27"), (r28, "28");
            (f0, "0"), (f1, "1"), (f2, "2"), (f3, "3"), (f4, "4"), (f5, "5"), (f6, "6"), (f7, "7");
            (f8, "8"), (f9, "9"), (f10, "10"), (f11, "11"), (f12, "12"), (f13, "13"), (f14, "14"), (f15, "15");
            (f16, "16"), (f17, "17"), (f18, "18"), (f19, "19"), (f20, "20"), (f21, "21"), (f22, "22"), (f23, "23");
            (f24, "24"), (f25, "25"), (f26, "26"), (f27, "27"), (f28, "28"), (f29, "29"), (f30, "30"), (f31, "31");
            (cr, "cr");
            (cr0, "0"), (cr1, "1"), (cr2, "2"), (cr3, "3"), (cr4, "4"), (cr5, "5"), (cr6, "6"), (cr7, "7");
            (xer, "xer");
        }
    }

    pub fn overlapping_regs(self, mut cb: impl FnMut(PowerPCInlineAsmReg)) {
        macro_rules! reg_conflicts {
            (
                $(
                    $full:ident : $($field:ident)*
                ),*;
            ) => {
                match self {
                    $(
                        Self::$full => {
                            cb(Self::$full);
                            $(cb(Self::$field);)*
                        }
                        $(Self::$field)|* => {
                            cb(Self::$full);
                            cb(self);
                        }
                    )*
                    r => cb(r),
                }
            };
        }
        reg_conflicts! {
            cr : cr0 cr1 cr2 cr3 cr4 cr5 cr6 cr7;
        }
    }
}
