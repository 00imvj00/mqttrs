use bytes::{Buf, BytesMut, IntoBuf};
use std::{io, num::NonZeroU16};

/// Packet Identifier, for ack purposes.
///
/// Note that the spec disallows a pid of 0 ([MQTT-2.3.1-1] for mqtt3, [MQTT-2.2.1-3] for mqtt5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PacketIdentifier(NonZeroU16);
impl PacketIdentifier {
    pub fn new(u: u16) -> Result<Self, io::Error> {
        match NonZeroU16::new(u) {
            Some(nz) => Ok(PacketIdentifier(nz)),
            None => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Zero PacketIdentifier",
            )),
        }
    }
    pub fn get(self) -> u16 {
        self.0.get()
    }
}

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

impl ConnectReturnCode {
    pub fn to_u8(&self) -> u8 {
        match *self {
            ConnectReturnCode::Accepted => 0,
            ConnectReturnCode::RefusedProtocolVersion => 1,
            ConnectReturnCode::RefusedIdentifierRejected => 2,
            ConnectReturnCode::ServerUnavailable => 3,
            ConnectReturnCode::BadUsernamePassword => 4,
            ConnectReturnCode::NotAuthorized => 5,
        }
    }

    pub fn from_u8(byte: u8) -> Result<ConnectReturnCode, io::Error> {
        match byte {
            0 => Ok(ConnectReturnCode::Accepted),
            1 => Ok(ConnectReturnCode::RefusedProtocolVersion),
            2 => Ok(ConnectReturnCode::RefusedIdentifierRejected),
            3 => Ok(ConnectReturnCode::ServerUnavailable),
            4 => Ok(ConnectReturnCode::BadUsernamePassword),
            5 => Ok(ConnectReturnCode::NotAuthorized),
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "")),
        }
    }
}

pub fn read_string(buffer: &mut BytesMut) -> String {
    let length = buffer.split_to(2).into_buf().get_u16_be();
    let byts = buffer.split_to(length as usize);
    return String::from_utf8(byts.to_vec()).unwrap().to_string();
}
