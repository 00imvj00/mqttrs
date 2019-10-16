use crate::{PacketType, QoS};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Header {
    hd: u8,
    packet_type: PacketType,
    len: usize,
}

impl Header {
    pub fn new(hd: u8, len: usize) -> Result<Header, io::Error> {
        Ok(Header {
            hd,
            len,
            packet_type: PacketType::from_hd(hd)?,
        })
    }
    pub fn packet(&self) -> PacketType {
        self.packet_type
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline]
    pub fn dup(&self) -> bool {
        (self.hd & 0b1000) != 0
    }
    #[inline]
    pub fn qos(&self) -> Result<QoS, io::Error> {
        QoS::from_hd(self.hd)
    }
    #[inline]
    pub fn retain(&self) -> bool {
        (self.hd & 1) != 0
    }
}

/* TESTS */
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn header() {
        let h = Header::new(0b00010000, 0).unwrap();
        assert_eq!(h.packet(), PacketType::Connect)
    }
}
