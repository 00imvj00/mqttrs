use crate::{utils, ConnectReturnCode, LastWill, Protocol, QoS};
use bytes::{Buf, BytesMut, IntoBuf};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct Connect {
    pub protocol: Protocol,
    pub keep_alive: u16,
    pub client_id: String,
    pub clean_session: bool,
    pub last_will: Option<LastWill>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Connack {
    pub session_present: bool,
    pub code: ConnectReturnCode,
}

impl Connect {
    pub fn from_buffer(buffer: &mut BytesMut) -> Result<Self, io::Error> {
        let protocol_name = utils::read_string(buffer);
        let protocol_level = buffer.split_to(1).into_buf().get_u8();
        let protocol = Protocol::new(&protocol_name, protocol_level).unwrap();

        let connect_flags = buffer.split_to(1).into_buf().get_u8();
        let keep_alive = buffer.split_to(2).into_buf().get_u16_be();

        let client_id = utils::read_string(buffer);

        let last_will = if connect_flags & 0b100 != 0 {
            let will_topic = utils::read_string(buffer);
            let will_message = utils::read_string(buffer);
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
            Some(utils::read_string(buffer))
        } else {
            None
        };

        let password = if connect_flags & 0b01000000 != 0 {
            Some(utils::read_string(buffer))
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
}
