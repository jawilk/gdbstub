//! Implementations for the BPF architecture.

use gdbstub::arch::{Arch, SingleStepGdbBehavior};

pub mod reg;

/// BPF-specific breakpoint kinds.
///
/// Extracted from the GDB source code [BPF Breakpoint Kinds](https://github.com/bminor/binutils-gdb/blob/9e0f6329352ab9c5e2f278181a3875918cff3b27/gdb/bpf-tdep.c#L205)
#[derive(Debug)]
pub enum BpfBreakpointKind {
    /// BPF breakpoint
    BpfBpKindBrkpt,
}

impl gdbstub::arch::BreakpointKind for BpfBreakpointKind {
    fn from_usize(kind: usize) -> Option<Self> {
        let kind = match kind {
            0 => BpfBreakpointKind::BpfBpKindBrkpt,
            _ => return None,
        };
        Some(kind)
    }
}

/// Implements `Arch` for 32-bit BPF.
pub enum Bpf {}

/// Implements `Arch` for 64-bit BPF.
pub enum Bpf64 {}

#[allow(deprecated)]
impl Arch for Bpf {
    type Usize = u32;
    type Registers = reg::BpfRegs<u32>;
    type RegId = reg::id::BpfRegId<u32>;
    type BreakpointKind = BpfBreakpointKind;

    fn target_description_xml() -> Option<&'static str> {
        None
    }

    #[inline(always)]
    fn single_step_gdb_behavior() -> SingleStepGdbBehavior {
        SingleStepGdbBehavior::Required
    }
}

#[allow(deprecated)]
impl Arch for Bpf64 {
    type Usize = u64;
    type Registers = reg::BpfRegs<u64>;
    type RegId = reg::id::BpfRegId<u64>;
    type BreakpointKind = BpfBreakpointKind;

    fn target_description_xml() -> Option<&'static str> {
        None
    }

    #[inline(always)]
    fn single_step_gdb_behavior() -> SingleStepGdbBehavior {
        SingleStepGdbBehavior::Required
    }
}
