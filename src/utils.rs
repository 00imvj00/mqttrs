use crate::encoder::check_remaining;
use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use std::{
    io::{Error, ErrorKind},
    num::NonZeroU16,
};

/// Packet Identifier, for ack purposes.
///
/// The spec ([MQTT-2.3.1-1], [MQTT-2.2.1-3]) disallows a pid of 0.
///
/// [MQTT-2.3.1-1]: https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718025
/// [MQTT-2.2.1-3]: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901026
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PacketIdentifier(NonZeroU16);
impl PacketIdentifier {
    pub fn new(u: u16) -> Result<Self, Error> {
        match NonZeroU16::new(u) {
            Some(nz) => Ok(PacketIdentifier(nz)),
            None => Err(Error::new(ErrorKind::InvalidData, "Pid == 0")),
        }
    }
    pub fn get(self) -> u16 {
        self.0.get()
    }
    pub(crate) fn from_buffer(buf: &mut BytesMut) -> Result<Self, Error> {
        Self::new(buf.split_to(2).into_buf().get_u16_be())
    }
    pub(crate) fn to_buffer(self, buf: &mut BytesMut) -> Result<(), Error> {
        check_remaining(buf, 2)?;
        Ok(buf.put_u16_be(self.get()))
    }
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
    pub fn from_u8(byte: u8) -> Result<QoS, Error> {
        match byte {
            0 => Ok(QoS::AtMostOnce),
            1 => Ok(QoS::AtLeastOnce),
            2 => Ok(QoS::ExactlyOnce),
            _ => Err(Error::new(ErrorKind::InvalidData, "Qos > 2")),
        }
    }
    #[inline]
    pub(crate) fn from_hd(hd: u8) -> Result<QoS, Error> {
        Self::from_u8((hd & 0b110) >> 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QosPid {
    AtMostOnce,
    AtLeastOnce(PacketIdentifier),
    ExactlyOnce(PacketIdentifier),
}
impl QosPid {
    pub fn from_u8u16(qos: u8, pid: u16) -> Result<Self, Error> {
        match qos {
            0 => Ok(QosPid::AtMostOnce),
            1 => Ok(QosPid::AtLeastOnce(PacketIdentifier::new(pid)?)),
            2 => Ok(QosPid::ExactlyOnce(PacketIdentifier::new(pid)?)),
            _ => Err(Error::new(ErrorKind::InvalidData, "Qos > 2")),
        }
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
    pub message: Vec<u8>,
    pub qos: QoS,
    pub retain: bool,
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

    pub fn from_u8(byte: u8) -> Result<ConnectReturnCode, Error> {
        match byte {
            0 => Ok(ConnectReturnCode::Accepted),
            1 => Ok(ConnectReturnCode::RefusedProtocolVersion),
            2 => Ok(ConnectReturnCode::RefusedIdentifierRejected),
            3 => Ok(ConnectReturnCode::ServerUnavailable),
            4 => Ok(ConnectReturnCode::BadUsernamePassword),
            5 => Ok(ConnectReturnCode::NotAuthorized),
            _ => Err(Error::new(ErrorKind::InvalidInput, "ConnectReturnCode > 5")),
        }
    }
}
