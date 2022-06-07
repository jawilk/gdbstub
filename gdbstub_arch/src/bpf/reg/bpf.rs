use core::convert::TryInto;

use num_traits::PrimInt;

use gdbstub::arch::Registers;
use gdbstub::internal::LeBytes;

/// BPF registers.
///
/// The register width of the SP and PC register is set to `u32` or `u64` based on the `<U>` type.
///
/// Source: <https://github.com/bminor/binutils-gdb/blob/9e0f6329352ab9c5e2f278181a3875918cff3b27/gdb/bpf-tdep.c#L42>
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct BpfRegs<U> {
    /// General purpose registers (R0-R9)
    pub r: [u64; 10],
    /// Stack pointer (R10)
    pub sp: U,
    /// Program counter (R11)
    pub pc: U,
}

impl<U> Registers for BpfRegs<U>
where
    U: PrimInt + LeBytes + Default + core::fmt::Debug,
{
    type ProgramCounter = U;

    fn pc(&self) -> Self::ProgramCounter {
        self.pc
    }

    fn gdb_serialize(&self, mut write_byte: impl FnMut(Option<u8>)) {
        macro_rules! write_le_bytes {
            ($value:expr) => {
                let mut buf = [0; 16];
                // infallible (unless digit is a >128 bit number)
                let len = $value.to_le_bytes(&mut buf).unwrap();
                let buf = &buf[..len];
                for b in buf {
                    write_byte(Some(*b));
                }
            };
        }

        // Write GPRs
       for reg in self.r.iter() {
           for b in reg.to_le_bytes() {
                write_byte(Some(b));
           }
       }

        // Write stack pointer register
        write_le_bytes!(&self.sp);
        // Write program counter register
        write_le_bytes!(&self.pc);
    }

    fn gdb_deserialize(&mut self, mut bytes: &[u8]) -> Result<(), ()> {
        let ptrsize = core::mem::size_of::<U>();

        // Ensure bytes contains enough data for all 12 registers
        if bytes.len() < ((8 * 10) + (ptrsize * 2)) {
            return Err(());
        }

        let mut regs = bytes[0..0x50]
            .chunks_exact(8)
            .map(|x| u64::from_be_bytes(x.try_into().unwrap()));

        // Read general purpose register
        for reg in self.r.iter_mut() {
            *reg = regs.next().ok_or(())?;
        }

        // Calculate the offset to the end of the registers based on the ptrsize
        let end_regs = 0x50 + (ptrsize * 2);

        // Read stack pointer and program counter
	let mut regs = bytes[0x50..end_regs]
	    .chunks_exact(ptrsize)
	    .map(|c| U::from_le_bytes(c).unwrap());

	self.sp = regs.next().ok_or(())?;
        self.pc = regs.next().ok_or(())?;

        Ok(())
    }
}
