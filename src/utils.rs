use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use std::{
    error::Error as ErrorTrait,
    fmt,
    io::{Error as IoError, ErrorKind},
    num::NonZeroU16,
};

/// Errors returned by [`encode()`] and [`decode()`].
///
/// [`encode()`]: fn.encode.html
/// [`decode()`]: fn.decode.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Not enough data in the read buffer.
    ///
    /// Do not treat this as a fatal error. Read more data into the buffer and try `decode()` again.
    UnexpectedEof,
    /// Not enough space in the write buffer.
    ///
    /// It is the caller's responsiblity to pass a big enough buffer to `encode()`.
    WriteZero,
    /// Tried to encode or decode a ProcessIdentifier==0.
    InvalidPid,
    /// Tried to decode a QoS > 2.
    InvalidQos(u8),
    /// Tried to decode a ConnectReturnCode > 5.
    InvalidConnectReturnCode(u8),
    /// Tried to decode an unknown protocol.
    InvalidProtocol(String, u8),
    /// Tried to decode an invalid packet type for this protocol.
    InvalidPacket(u8),
    /// Trying to encode/decode an invalid length.
    ///
    /// The difference with `BufferFull`/`BufferIncomplete` is that it refers to an invalid/corrupt
    /// length rather than a buffer size issue.
    InvalidLength(usize),
    /// Trying to decode a non-utf8 string.
    InvalidString(std::str::Utf8Error),
    /// Catch-all error when converting from `std::io::Error`.
    ///
    /// You'll hopefully never see this.
    IoError(ErrorKind, String),
}
impl ErrorTrait for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl From<Error> for IoError {
    fn from(err: Error) -> IoError {
        match err {
            Error::WriteZero => IoError::new(ErrorKind::WriteZero, err),
            Error::UnexpectedEof => IoError::new(ErrorKind::UnexpectedEof, err),
            _ => IoError::new(ErrorKind::InvalidData, err),
        }
    }
}
impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        match err.kind() {
            ErrorKind::WriteZero => Error::WriteZero,
            ErrorKind::UnexpectedEof => Error::UnexpectedEof,
            k => Error::IoError(k, format!("{}",err)),
        }
    }
}

/// Packet Identifier.
///
/// For packets with [`QoS::AtLeastOne` or `QoS::ExactlyOnce`] delivery.
///
/// ```rust
/// # use mqttrs::{Pid, Packet};
/// let pid = Pid::new(42).expect("illegal pid value");
/// let next_pid = pid + 1;
/// let pending_acks = std::collections::HashMap::<Pid, Packet>::new();
/// ```
///
/// The spec ([MQTT-2.3.1-1], [MQTT-2.2.1-3]) disallows a pid of 0.
///
/// [`QoS::AtLeastOne` or `QoS::ExactlyOnce`]: enum.QoS.html
/// [MQTT-2.3.1-1]: https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718025
/// [MQTT-2.2.1-3]: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901026
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pid(NonZeroU16);
impl Pid {
    pub fn new(u: u16) -> Result<Self, Error> {
        match NonZeroU16::new(u) {
            Some(nz) => Ok(Pid(nz)),
            None => Err(Error::InvalidPid),
        }
    }
    pub fn get(self) -> u16 {
        self.0.get()
    }
    pub(crate) fn from_buffer(buf: &mut BytesMut) -> Result<Self, Error> {
        Self::new(buf.split_to(2).into_buf().get_u16_be())
    }
    pub(crate) fn to_buffer(self, buf: &mut BytesMut) -> Result<(), Error> {
        Ok(buf.put_u16_be(self.get()))
    }
}
impl std::ops::Add<u16> for Pid {
    type Output = Pid;
    fn add(self, u: u16) -> Pid {
        let n = self.get().wrapping_add(u);
        Pid(NonZeroU16::new(if n == 0 { 1 } else { n }).unwrap())
    }
}
impl std::ops::Sub<u16> for Pid {
    type Output = Pid;
    fn sub(self, u: u16) -> Pid {
        let n = self.get().wrapping_sub(u);
        Pid(NonZeroU16::new(if n == 0 { std::u16::MAX } else { n }).unwrap())
    }
}

/// Packet delivery [Quality of Service] level.
///
/// [Quality of Service]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718099
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QoS {
    /// `QoS 0`. No ack needed.
    AtMostOnce,
    /// `QoS 1`. One ack needed.
    AtLeastOnce,
    /// `QoS 2`. Two acks needed.
    ExactlyOnce,
}
impl QoS {
    pub(crate) fn to_u8(&self) -> u8 {
        match *self {
            QoS::AtMostOnce => 0,
            QoS::AtLeastOnce => 1,
            QoS::ExactlyOnce => 2,
        }
    }
    pub(crate) fn from_u8(byte: u8) -> Result<QoS, Error> {
        match byte {
            0 => Ok(QoS::AtMostOnce),
            1 => Ok(QoS::AtLeastOnce),
            2 => Ok(QoS::ExactlyOnce),
            n => Err(Error::InvalidQos(n)),
        }
    }
    #[inline]
    pub(crate) fn from_hd(hd: u8) -> Result<QoS, Error> {
        Self::from_u8((hd & 0b110) >> 1)
    }
}

/// Combined [`QoS`]/[`Pid`].
///
/// Used only in [`Publish`] packets.
///
/// [`Publish`]: struct.Publish.html
/// [`QoS`]: enum.QoS.html
/// [`Pid`]: struct.Pid.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QosPid {
    AtMostOnce,
    AtLeastOnce(Pid),
    ExactlyOnce(Pid),
}
impl QosPid {
    #[cfg(test)]
    pub(crate) fn from_u8u16(qos: u8, pid: u16) -> Self {
        match qos {
            0 => QosPid::AtMostOnce,
            1 => QosPid::AtLeastOnce(Pid::new(pid).expect("pid == 0")),
            2 => QosPid::ExactlyOnce(Pid::new(pid).expect("pid == 0")),
            _ => panic!("Qos > 2"),
        }
    }
    /// Extract the [`Pid`] from a `QosPid`, if any.
    ///
    /// [`Pid`]: struct.Pid.html
    pub fn pid(self) -> Option<Pid> {
        match self {
            QosPid::AtMostOnce => None,
            QosPid::AtLeastOnce(p) => Some(p),
            QosPid::ExactlyOnce(p) => Some(p),
        }
    }
    /// Extract the [`QoS`] from a `QosPid`.
    ///
    /// [`QoS`]: enum.QoS.html
    pub fn qos(self) -> QoS {
        match self {
            QosPid::AtMostOnce => QoS::AtMostOnce,
            QosPid::AtLeastOnce(_) => QoS::AtLeastOnce,
            QosPid::ExactlyOnce(_) => QoS::ExactlyOnce,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LastWill {
    pub topic: String,
    pub message: Vec<u8>,
    pub qos: QoS,
    pub retain: bool,
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
impl ConnectReturnCode {
    pub(crate) fn to_u8(&self) -> u8 {
        match *self {
            ConnectReturnCode::Accepted => 0,
            ConnectReturnCode::RefusedProtocolVersion => 1,
            ConnectReturnCode::RefusedIdentifierRejected => 2,
            ConnectReturnCode::ServerUnavailable => 3,
            ConnectReturnCode::BadUsernamePassword => 4,
            ConnectReturnCode::NotAuthorized => 5,
        }
    }
    pub(crate) fn from_u8(byte: u8) -> Result<ConnectReturnCode, Error> {
        match byte {
            0 => Ok(ConnectReturnCode::Accepted),
            1 => Ok(ConnectReturnCode::RefusedProtocolVersion),
            2 => Ok(ConnectReturnCode::RefusedIdentifierRejected),
            3 => Ok(ConnectReturnCode::ServerUnavailable),
            4 => Ok(ConnectReturnCode::BadUsernamePassword),
            5 => Ok(ConnectReturnCode::NotAuthorized),
            n => Err(Error::InvalidConnectReturnCode(n)),
        }
    }
}
