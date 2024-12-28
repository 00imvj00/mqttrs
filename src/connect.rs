#[cfg(feature = "defmt")]
use defmt::Format;
use crate::{decoder::*, encoder::*, *};
#[cfg(not(feature = "std"))]
use heapless::String;
#[cfg(feature = "std")]
use std::string::String;
use core::str::FromStr;


/// Protocol version.
///
/// Sent in [`Connect`] packet.
///
/// [`Connect`]: struct.Connect.html
///
#[cfg_attr(feature = "defmt",derive(Format))]
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
            _ => Err(Error::InvalidProtocol(String::from_str(name).unwrap(), 0)),
    }
    }
    pub(crate) fn from_buffer<'a>(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let protocol_name = read_str(buf, offset)?;
        let protocol_level = buf[*offset];
        *offset += 1;

        Protocol::new(protocol_name, protocol_level)
    }
    pub(crate) fn to_buffer(&self, buf: &mut [u8], offset: &mut usize) -> Result<usize, Error> {
        match self {
            Protocol::MQTT311 => {
                let slice = &[0u8, 4, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 4];
                for &byte in slice {
                    write_u8(buf, offset, byte)?;
                }
                Ok(slice.len())
            }
            Protocol::MQIsdp => {
                let slice = &[
                    0u8, 4, 'M' as u8, 'Q' as u8, 'i' as u8, 's' as u8, 'd' as u8, 'p' as u8, 4,
                ];
                for &byte in slice {
                    write_u8(buf, offset, byte)?;
                }
                Ok(slice.len())
            }
        }
    }
}

/// Message that the server should publish when the client disconnects.
///
/// Sent by the client in the [Connect] packet. [MQTT 3.1.3.3].
///
/// [Connect]: struct.Connect.html
/// [MQTT 3.1.3.3]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718031
#[cfg_attr(feature = "defmt",derive(Format))]
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
#[cfg_attr(feature = "defmt",derive(Format))]
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
#[cfg_attr(feature = "defmt",derive(Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct Connect<'a> {
    pub protocol: Protocol,
    pub keep_alive: u16,
    pub client_id: &'a str,
    pub clean_session: bool,
    pub last_will: Option<LastWill<'a>>,
    pub username: Option<&'a str>,
    pub password: Option<&'a [u8]>,
}

/// Connack packet ([MQTT 3.2]).
///
/// [MQTT 3.2]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718033
#[cfg_attr(feature = "defmt",derive(Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Connack {
    pub session_present: bool,
    pub code: ConnectReturnCode,
}

impl<'a> Connect<'a> {
    pub(crate) fn from_buffer(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let protocol = Protocol::from_buffer(buf, offset)?;

        let connect_flags = buf[*offset];
        let keep_alive = ((buf[*offset + 1] as u16) << 8) | buf[*offset + 2] as u16;
        *offset += 3;

        let client_id = read_str(buf, offset)?;

        let last_will = if connect_flags & 0b100 != 0 {
            let will_topic = read_str(buf, offset)?;
            let will_message = read_bytes(buf, offset)?;
            let will_qod = QoS::from_u8((connect_flags & 0b11000) >> 3)?;
            Some(LastWill {
                topic: will_topic,
                message: will_message,
                qos: will_qod,
                retain: (connect_flags & 0b00100000) != 0,
            })
        } else {
            None
        };

        let username = if connect_flags & 0b10000000 != 0 {
            Some(read_str(buf, offset)?)
        } else {
            None
        };

        let password = if connect_flags & 0b01000000 != 0 {
            Some(read_bytes(buf, offset)?)
        } else {
            None
        };

        let clean_session = (connect_flags & 0b10) != 0;

        Ok(Connect {
            protocol,
            keep_alive,
            client_id,
            username,
            password,
            last_will,
            clean_session,
        })
    }

    pub(crate) fn to_buffer(&self, buf: &mut [u8], offset: &mut usize) -> Result<usize, Error> {
        let header: u8 = 0b00010000;
        let mut length: usize = 6 + 1 + 1; // NOTE: protocol_name(6) + protocol_level(1) + flags(1);
        let mut connect_flags: u8 = 0b00000000;
        if self.clean_session {
            connect_flags |= 0b10;
        };
        length += 2 + self.client_id.len();
        length += 2; // keep alive
        if let Some(username) = self.username {
            connect_flags |= 0b10000000;
            length += username.len();
            length += 2;
        };
        if let Some(password) = self.password {
            connect_flags |= 0b01000000;
            length += password.len();
            length += 2;
        };
        if let Some(last_will) = &self.last_will {
            connect_flags |= 0b00000100;
            connect_flags |= last_will.qos.to_u8() << 3;
            if last_will.retain {
                connect_flags |= 0b00100000;
            };
            length += last_will.message.len();
            length += last_will.topic.len();
            length += 4;
        };
        check_remaining(buf, offset, length + 1)?;

        // NOTE: putting data into buffer.
        write_u8(buf, offset, header)?;

        let write_len = write_length(buf, offset, length)? + 1;
        self.protocol.to_buffer(buf, offset)?;

        write_u8(buf, offset, connect_flags)?;
        write_u16(buf, offset, self.keep_alive)?;

        write_string(buf, offset, self.client_id)?;

        if let Some(last_will) = &self.last_will {
            write_string(buf, offset, last_will.topic)?;
            write_bytes(buf, offset, &last_will.message)?;
        };

        if let Some(username) = self.username {
            write_string(buf, offset, username)?;
        };
        if let Some(password) = self.password {
            write_bytes(buf, offset, password)?;
        };
        // NOTE: END
        Ok(write_len)
    }
}

impl Connack {
    pub(crate) fn from_buffer<'a>(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let flags = buf[*offset];
        let return_code = buf[*offset + 1];
        *offset += 2;
        Ok(Connack {
            session_present: (flags & 0b1 == 1),
            code: ConnectReturnCode::from_u8(return_code)?,
        })
    }
    pub(crate) fn to_buffer(&self, buf: &mut [u8], offset: &mut usize) -> Result<usize, Error> {
        check_remaining(buf, offset, 4)?;
        let header: u8 = 0b00100000;
        let length: u8 = 2;
        let mut flags: u8 = 0b00000000;
        if self.session_present {
            flags |= 0b1;
        };
        let rc = self.code.to_u8();
        write_u8(buf, offset, header)?;
        write_u8(buf, offset, length)?;
        write_u8(buf, offset, flags)?;
        write_u8(buf, offset, rc)?;
        Ok(4)
    }
}
