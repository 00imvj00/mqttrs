use crate::QoS;
use std::io;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PacketType {
    Connect,
    Connack,
    Publish,
    Puback,
    Pubrec,
    Pubrel,
    PubComp,
    Subscribe,
    SubAck,
    UnSubscribe,
    UnSubAck,
    PingReq,
    PingResp,
    Disconnect,
}
impl PacketType {
    #[inline]
    pub fn from_hd(hd: u8) -> Result<PacketType, io::Error> {
        Self::from_u8(hd >> 4)
    }
    pub fn to_u8(&self) -> u8 {
        match *self {
            PacketType::Connect => 1,
            PacketType::Connack => 2,
            PacketType::Publish => 3,
            PacketType::Puback => 4,
            PacketType::Pubrec => 5,
            PacketType::Pubrel => 6,
            PacketType::PubComp => 7,
            PacketType::Subscribe => 8,
            PacketType::SubAck => 9,
            PacketType::UnSubscribe => 10,
            PacketType::UnSubAck => 11,
            PacketType::PingReq => 12,
            PacketType::PingResp => 13,
            PacketType::Disconnect => 14,
        }
    }
    pub fn from_u8(byte: u8) -> Result<Self, io::Error> {
        match byte {
            1 => Ok(PacketType::Connect),
            2 => Ok(PacketType::Connack),
            3 => Ok(PacketType::Publish),
            4 => Ok(PacketType::Puback),
            5 => Ok(PacketType::Pubrec),
            6 => Ok(PacketType::Pubrel),
            7 => Ok(PacketType::PubComp),
            8 => Ok(PacketType::Subscribe),
            9 => Ok(PacketType::SubAck),
            10 => Ok(PacketType::UnSubscribe),
            11 => Ok(PacketType::UnSubAck),
            12 => Ok(PacketType::PingReq),
            13 => Ok(PacketType::PingResp),
            14 => Ok(PacketType::Disconnect),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported packet type".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
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
