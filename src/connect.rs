use std::ops::Range;

use crate::errors::Error;
use crate::qos::QoS;
use bytes::BytesMut;
use std::string::String;

//use std::ops::Range;
/// Protocol version.
///
/// Sent in [`Connect`] packet.
///
/// [`Connect`]: struct.Connect.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// [MQTT 3.1.1] is the most commonly implemented version. [MQTT 5] isn't yet supported my by
    /// `mqttrs`.
    ///
    /// [MQTT 3.1.1]: https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html
    /// [MQTT 5]: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html
    MQTT311,
    /// MQIsdp, aka SCADA are pre-standardisation names of MQTT. It should mostly conform to MQTT
    /// 3.1.1, but you should watch out for implementation discrepancies. `Mqttrs` handles it like
    /// standard MQTT 3.1.1.
    MQIsdp,
}
impl Protocol {
    pub(crate) fn new(name: &str, level: u8) -> Result<Protocol, Error> {
        match (name, level) {
            ("MQIsdp", 3) => Ok(Protocol::MQIsdp),
            ("MQTT", 4) => Ok(Protocol::MQTT311),
            _ => Err(Error::InvalidProtocol(name.into(), level)),
        }
    }

    pub(crate) fn from(buf: &[u8], offset: usize) -> Result<Self, Error> {
        //* The length of the buffer should be more than 2, to get the length of protocol name strea
        if buf[offset..].len() < 2 {
            return Err(Error::InvalidLength);
        }

        let protocol_name_length = u16::from_be_bytes(buf[offset..(offset + 2)]);
        let protocol_name_slice = &[offset + 2..protocol_name_length];
        let protocol_name = std::str::from_utf8(protocol_name_slice);

        let protocol_level_byte_index = offset + 2 + protocol_name_slice.len();
        let protocol_level = buf[protocol_level_byte_index] as u8;

        Protocol::new(protocol_name, protocol_level)
    }
    //pub(crate) fn to_buffer(&self, buf: &mut [u8], offset: &mut usize) -> Result<usize, Error> {
    //match self {
    //Protocol::MQTT311 => {
    //let slice = &[0u8, 4, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 4];
    //for &byte in slice {
    //write_u8(buf, offset, byte)?;
    //}
    //Ok(slice.len())
    //}
    //Protocol::MQIsdp => {
    //let slice = &[
    //0u8, 4, 'M' as u8, 'Q' as u8, 'i' as u8, 's' as u8, 'd' as u8, 'p' as u8, 4,
    //];
    //for &byte in slice {
    //write_u8(buf, offset, byte)?;
    //}
    //Ok(slice.len())
    //}
    //}
    //}
}

/// Message that the server should publish when the client disconnects.
///
/// Sent by the client in the [Connect] packet. [MQTT 3.1.3.3].
///
/// [Connect]: struct.Connect.html
/// [MQTT 3.1.3.3]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718031
#[derive(Debug, Clone, PartialEq)]
pub struct LastWill<'a> {
    pub topic: &'a str,
    pub message: &'a [u8],
    pub qos: QoS,
    pub retain: bool,
}

/// Sucess value of a [Connack] packet.
///
/// See [MQTT 3.2.2.3] for interpretations.
///
/// [Connack]: struct.Connack.html
/// [MQTT 3.2.2.3]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718035
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
    fn to_u8(&self) -> u8 {
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

/// Connect packet ([MQTT 3.1]).
///
/// [MQTT 3.1]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718028
#[derive(Debug, Clone, PartialEq)]
pub struct Connect {
    buffer: bytes::BytesMut,
    protocol_range: Option<Range<usize>>,
    keep_alive: u16,
    client_id_range: Range<usize>,
    clean_session_range: bool,
    last_will_range: Option<Range<usize>>,
    username_range: Option<Range<usize>>,
    password_range: Option<Range<usize>>,
}

/// Connack packet ([MQTT 3.2]).
///
/// [MQTT 3.2]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718033
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Connack {
    pub session_present: bool,
    pub code: ConnectReturnCode,
}

impl Connect {
    pub fn new(stream: &[u8], size: usize) -> Result<Self, Error> {
        let mut pointer = 1;
        let protocol = Some(Range {
            start: pointer,
            end: pointer + 7,
        });
        pointer += 7;
        let connect_flags = stream[pointer] as u8;
        pointer += 1;
        let keep_alive = u16::from_be_bytes(&stream[pointer..pointer + 2]) as u16;
        pointer += 2;
        let client_id_length = u16::from_be_bytes(&stream[pointer..pointer + 2]) as u16;
        pointer += 2;
        let client_id = Range {
            start: pointer,
            end: pointer + client_id_length,
        };
        pointer += client_id_length;

        let clean_session = (connect_flags & 0b10) != 0;

        let last_will = if connect_flags & 0b100 != 0 {
            let start = pointer;
            let last_will_topic_length = u16::from_be_bytes(&stream[pointer..pointer + 2]) as u16;
            pointer += 2;
            pointer += last_will_topic_length;

            let last_will_message_length = u16::from_be_bytes(&stream[pointer..pointer + 2]) as u16;
            pointer += 2;
            pointer += last_will_message_length;

            Some(Range {
                start,
                end: pointer,
            })
        } else {
            None
        };

        let username = if connect_flags & 0b10000000 != 0 {
            let start = pointer;
            let username_len = u16::from_be_bytes(&stream[pointer..pointer + 2]) as u16;
            pointer += 2;
            pointer += username_len;
            Some(Range {
                start,
                end: pointer,
            })
        } else {
            None
        };

        let password = if connect_flags & 0b01000000 != 0 {
            let start = pointer;
            let password_len = u16::from_be_bytes(&stream[pointer..pointer + 2]) as u16;
            pointer += 2;
            pointer += password_len;
            Some(Range {
                start,
                end: pointer,
            })
        } else {
            None
        };

        let mut connect = Connect {
            buffer: BytesMut::with_capacity(size),
            protocol_range: protocol,
            keep_alive,
            client_id_range: client_id,
            clean_session_range: clean_session,
            username_range: username,
            password_range: password,
            last_will_range: last_will,
        };
        connect.buffer.put(&stream[0..size]);
        Ok(connect)
    }

    pub fn client_id(&self) -> Option<String> {
        let client_id_slice = &self.buffer[self.client_id_range.start..self.client_id_range.end];
        let id = String::from_utf8(client_id_slice);
        Some(id)
    }
}

//impl Connack {
//pub fn from(stream: &[u8]) -> Result<Self, Error> {
//let flags = stream[0];
//let session_present = flags & 0b00000001 == 1;
//let code = ConnectReturnCode::from_u8(stream[1]);
//let connack = Connack {
//session_present,
//code,
//};
//Ok(connack)
//}

//pub fn to(stream: &mut [u8]) -> Result<(), Error> {
//todo!()
//}
//}
