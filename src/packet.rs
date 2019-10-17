use crate::{Connack, Connect, PacketIdentifier, Publish, Suback, Subscribe, Unsubscribe};
use std::io::{Error, ErrorKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    Connect(Connect),
    Connack(Connack),
    Publish(Publish),
    Puback(PacketIdentifier),
    Pubrec(PacketIdentifier),
    Pubrel(PacketIdentifier),
    PubComp(PacketIdentifier),
    Subscribe(Subscribe),
    SubAck(Suback),
    UnSubscribe(Unsubscribe),
    UnSubAck(PacketIdentifier),
    PingReq,
    PingResp,
    Disconnect,
}
impl Packet {
    pub fn get_type(&self) -> PacketType {
        match self {
            Packet::Connect(_) => PacketType::Connect,
            Packet::Connack(_) => PacketType::Connack,
            Packet::Publish(_) => PacketType::Publish,
            Packet::Puback(_) => PacketType::Puback,
            Packet::Pubrec(_) => PacketType::Pubrec,
            Packet::Pubrel(_) => PacketType::Pubrel,
            Packet::PubComp(_) => PacketType::PubComp,
            Packet::Subscribe(_) => PacketType::Subscribe,
            Packet::SubAck(_) => PacketType::SubAck,
            Packet::UnSubscribe(_) => PacketType::UnSubscribe,
            Packet::UnSubAck(_) => PacketType::UnSubAck,
            Packet::PingReq => PacketType::PingReq,
            Packet::PingResp => PacketType::PingResp,
            Packet::Disconnect => PacketType::Disconnect,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    pub(crate) fn from_hd(hd: u8) -> Result<Self, Error> {
        match hd >> 4 {
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
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "Unsupported packet type",
            )),
        }
    }
}
