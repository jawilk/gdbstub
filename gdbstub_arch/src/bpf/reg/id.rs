use core::num::NonZeroUsize;

use gdbstub::arch::RegId;

/// BPF register identifier.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum BpfRegId<U> {
    /// General purpose registers (R0-R9)
    Gpr(u8),
    /// Stack Pointer (R10)
    Sp,
    /// Program Counter (R11)
    Pc,
    #[doc(hidden)]
    _Size(core::marker::PhantomData<U>),
}

fn from_raw_id<U>(id: usize) -> Option<(BpfRegId<U>, Option<NonZeroUsize>)> {
    let reg = match id {
        // `BpfRegId::Gpr` register are 8 bytes wide
        0..=9 => return Some((BpfRegId::Gpr(id as u8), Some(NonZeroUsize::new(8)?))),
        10 => BpfRegId::Sp,
        11 => BpfRegId::Pc,
        _ => return None,
    };

    let ptrsize = core::mem::size_of::<U>();
    Some((reg, Some(NonZeroUsize::new(ptrsize)?)))
}

impl RegId for BpfRegId<u32> {
    fn from_raw_id(id: usize) -> Option<(Self, Option<NonZeroUsize>)> {
        from_raw_id::<u32>(id)
    }
}

impl RegId for BpfRegId<u64> {
    fn from_raw_id(id: usize) -> Option<(Self, Option<NonZeroUsize>)> {
        from_raw_id::<u64>(id)
    }
}
