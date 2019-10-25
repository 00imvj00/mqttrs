use crate::{decoder::*, encoder::*, *};
use bytes::{Buf, BufMut, BytesMut, IntoBuf};

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
    pub(crate) fn to_buffer(&self, buffer: &mut BytesMut) -> Result<(), Error> {
        match self {
            Protocol::MQTT311 => {
                Ok(buffer.put_slice(&[0u8, 4, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 4]))
            }
            Protocol::MQIsdp => Ok(buffer.put_slice(&[
                0u8, 4, 'M' as u8, 'Q' as u8, 'i' as u8, 's' as u8, 'd' as u8, 'p' as u8, 4,
            ])),
        }
    }
}

/// Message that the server should publish when the client disconnects.
///
/// Sent by the client in the [Connect] packet. [MQTT 3.1.3.3].
///
/// [Connect]: struct.Connect.html
/// [MQTT 3.1.3.3]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718031
#[derive(Debug, Clone, PartialEq)]
pub struct LastWill {
    pub topic: String,
    pub message: Vec<u8>,
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
    pub protocol: Protocol,
    pub keep_alive: u16,
    pub client_id: String,
    pub clean_session: bool,
    pub last_will: Option<LastWill>,
    pub username: Option<String>,
    pub password: Option<Vec<u8>>,
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
    pub(crate) fn from_buffer(buffer: &mut BytesMut) -> Result<Self, Error> {
        let protocol_name = read_string(buffer)?;
        let protocol_level = buffer.split_to(1).into_buf().get_u8();
        let protocol = Protocol::new(&protocol_name, protocol_level).unwrap();

        let connect_flags = buffer.split_to(1).into_buf().get_u8();
        let keep_alive = buffer.split_to(2).into_buf().get_u16_be();

        let client_id = read_string(buffer)?;

        let last_will = if connect_flags & 0b100 != 0 {
            let will_topic = read_string(buffer)?;
            let will_message = read_bytes(buffer)?;
            let will_qod = QoS::from_u8((connect_flags & 0b11000) >> 3).unwrap();
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
            Some(read_string(buffer)?)
        } else {
            None
        };

        let password = if connect_flags & 0b01000000 != 0 {
            Some(read_bytes(buffer)?)
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
    pub(crate) fn to_buffer(&self, buffer: &mut BytesMut) -> Result<(), Error> {
        let header_u8: u8 = 0b00010000;
        let mut length: usize = 6 + 1 + 1; //NOTE: protocol_name(6) + protocol_level(1) + flags(1);
        let mut connect_flags: u8 = 0b00000000;
        if self.clean_session {
            connect_flags |= 0b10;
        };
        length += 2 + self.client_id.len();
        length += 2; //keep alive
        if let Some(username) = &self.username {
            connect_flags |= 0b10000000;
            length += username.len();
            length += 2;
        };
        if let Some(password) = &self.password {
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
        check_remaining(buffer, length + 1)?;

        //NOTE: putting data into buffer.
        buffer.put(header_u8);
        write_length(length, buffer)?;
        self.protocol.to_buffer(buffer)?;
        buffer.put(connect_flags);
        buffer.put_u16_be(self.keep_alive);
        write_string(self.client_id.as_ref(), buffer)?;

        if let Some(last_will) = &self.last_will {
            write_string(last_will.topic.as_ref(), buffer)?;
            write_bytes(&last_will.message, buffer)?;
        };

        if let Some(username) = &self.username {
            write_string(username.as_ref(), buffer)?;
        };
        if let Some(password) = &self.password {
            write_bytes(password, buffer)?;
        };
        //NOTE: END
        Ok(())
    }
}

impl Connack {
    pub(crate) fn from_buffer(buffer: &mut BytesMut) -> Result<Self, Error> {
        let flags = buffer.split_to(1).into_buf().get_u8();
        let return_code = buffer.split_to(1).into_buf().get_u8();
        Ok(Connack {
            session_present: (flags & 0b1 == 1),
            code: ConnectReturnCode::from_u8(return_code)?,
        })
    }
    pub(crate) fn to_buffer(&self, buffer: &mut BytesMut) -> Result<(), Error> {
        check_remaining(buffer, 4)?;
        let header_u8 = 0b00100000 as u8;
        let length = 2 as u8;
        let mut flags = 0b00000000 as u8;
        if self.session_present {
            flags |= 0b1;
        };
        let rc = self.code.to_u8();
        buffer.put(header_u8);
        buffer.put(length);
        buffer.put(flags);
        buffer.put(rc);
        Ok(())
    }
}
