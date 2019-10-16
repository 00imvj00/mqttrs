use crate::{decoder::*, encoder::*, ConnectReturnCode, LastWill, Protocol, QoS};
use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use std::io;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Connack {
    pub session_present: bool,
    pub code: ConnectReturnCode,
}

impl Connect {
    pub(crate) fn from_buffer(buffer: &mut BytesMut) -> Result<Self, io::Error> {
        let protocol_name = read_string(buffer);
        let protocol_level = buffer.split_to(1).into_buf().get_u8();
        let protocol = Protocol::new(&protocol_name, protocol_level).unwrap();

        let connect_flags = buffer.split_to(1).into_buf().get_u8();
        let keep_alive = buffer.split_to(2).into_buf().get_u16_be();

        let client_id = read_string(buffer);

        let last_will = if connect_flags & 0b100 != 0 {
            let will_topic = read_string(buffer);
            let will_message = read_bytes(buffer);
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
            Some(read_string(buffer))
        } else {
            None
        };

        let password = if connect_flags & 0b01000000 != 0 {
            Some(read_bytes(buffer))
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
    pub(crate) fn to_buffer(&self, buffer: &mut BytesMut) -> Result<(), io::Error> {
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
    pub(crate) fn from_buffer(buffer: &mut BytesMut) -> Result<Self, io::Error> {
        let flags = buffer.split_to(1).into_buf().get_u8();
        let return_code = buffer.split_to(1).into_buf().get_u8();
        Ok(Connack {
            session_present: (flags & 0b1 == 1),
            code: ConnectReturnCode::from_u8(return_code)?,
        })
    }
    pub(crate) fn to_buffer(&self, buffer: &mut BytesMut) -> Result<(), io::Error> {
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
