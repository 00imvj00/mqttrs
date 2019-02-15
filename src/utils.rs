use bytes::{Buf, BytesMut, IntoBuf};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PacketIdentifier(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    MQIsdp(u8),
    MQTT(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}
impl QoS {
    pub fn to_u8(&self) -> u8 {
        match *self {
            QoS::AtMostOnce => 0,
            QoS::AtLeastOnce => 1,
            QoS::ExactlyOnce => 2,
        }
    }
    pub fn from_u8(byte: u8) -> Result<QoS, io::Error> {
        match byte {
            0 => Ok(QoS::AtMostOnce),
            1 => Ok(QoS::AtLeastOnce),
            2 => Ok(QoS::ExactlyOnce),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "")),
        }
    }
    #[inline]
    pub fn from_hd(hd: u8) -> Result<QoS, io::Error> {
        Self::from_u8((hd & 0b110) >> 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectReturnCode {
    Accepted,
    RefusedProtocolVersion,
    RefusedIdentifierRejected,
    ServerUnavailable,
    BadUsernamePassword,
    NotAuthorized,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LastWill {
    pub topic: String,
    pub message: String,
    pub qos: QoS,
    pub retain: bool,
}

impl Protocol {
    pub fn new(name: &str, level: u8) -> Result<Protocol, io::Error> {
        match name {
            "MQIsdp" => match level {
                3 => Ok(Protocol::MQIsdp(3)),
                _ => Err(io::Error::new(io::ErrorKind::InvalidData, "")),
            },
            "MQTT" => match level {
                4 => Ok(Protocol::MQTT(4)),
                _ => Err(io::Error::new(io::ErrorKind::InvalidData, "")),
            },
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "")),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            &Protocol::MQIsdp(_) => "MQIsdp",
            &Protocol::MQTT(_) => "MQTT",
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            &Protocol::MQIsdp(level) => level,
            &Protocol::MQTT(level) => level,
        }
    }
}

pub fn read_string(buffer: &mut BytesMut) -> String {
    let length = buffer.split_to(2).into_buf().get_u16_be();
    let byts = buffer.split_to(length as usize);
    return String::from_utf8(byts.to_vec()).unwrap().to_string();
}
