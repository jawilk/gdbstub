
use super::prelude::*;

#[derive(Debug)]
pub struct qRegisterInfo {
    pub reg_num: u8,
}

impl<'a> ParseCommand<'a> for qRegisterInfo {
    fn from_packet(buf: PacketBuf<'a>) -> Option<Self> {
        crate::__dead_code_marker!("qRegisterInfo", "from_packet");
        let body = buf.into_body();
        if body.is_empty() {
            return None;
        }
        Some(qRegisterInfo {
            reg_num: u8::from_str_radix(core::str::from_utf8(body).unwrap(), 16).unwrap(),
	})
    }
}
