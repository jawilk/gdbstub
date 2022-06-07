//! `Register` structs for BPF architectures.

/// `RegId` definitions for BPF architectures.
pub mod id;

mod bpf;

pub use bpf::BpfRegs;
