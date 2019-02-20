use crate::{encoder, utils, PacketIdentifier, QoS};
use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct SubscribeTopic {
    pub topic_path: String,
    pub qos: QoS,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscribeReturnCodes {
    Success(QoS),
    Failure,
}
impl SubscribeReturnCodes {
    pub fn to_u8(&self) -> u8 {
        match *self {
            SubscribeReturnCodes::Failure => 0x80,
            SubscribeReturnCodes::Success(qos) => qos.to_u8(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Subscribe {
    pub pid: PacketIdentifier,
    pub topics: Vec<SubscribeTopic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Suback {
    pub pid: PacketIdentifier,
    pub return_codes: Vec<SubscribeReturnCodes>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unsubscribe {
    pub pid: PacketIdentifier,
    pub topics: Vec<String>,
}

impl Subscribe {
    pub fn from_buffer(buffer: &mut BytesMut) -> Result<Self, io::Error> {
        let pid = PacketIdentifier(buffer.split_to(2).into_buf().get_u16_be());
        let mut topics: Vec<SubscribeTopic> = Vec::new();
        while buffer.len() != 0 {
            let topic_path = utils::read_string(buffer);
            let qos = QoS::from_u8(buffer.split_to(1).into_buf().get_u8())?;
            let topic = SubscribeTopic { topic_path, qos };
            topics.push(topic);
        }
        Ok(Subscribe { pid, topics })
    }

    pub fn to_buffer(&self, buffer: &mut BytesMut) -> Result<(), io::Error> {
        let header_u8: u8 = 0b10000010;

        let mut length = 0;
        length += 2;

        for topic in &self.topics {
            length += topic.topic_path.len() + 2 + 1;
        }

        let PacketIdentifier(pid) = self.pid;

        buffer.put(header_u8);
        encoder::write_length(length, buffer)?;
        buffer.put_u16_be(pid as u16);

        for topic in &self.topics {
            encoder::write_string(topic.topic_path.as_ref(), buffer)?;
            buffer.put(topic.qos.to_u8());
        }

        Ok(())
    }
}

impl Unsubscribe {
    pub fn from_buffer(buffer: &mut BytesMut) -> Result<Self, io::Error> {
        let pid = PacketIdentifier(buffer.split_to(2).into_buf().get_u16_be());
        let mut topics: Vec<String> = Vec::new();
        while buffer.len() != 0 {
            let topic_path = utils::read_string(buffer);
            topics.push(topic_path);
        }
        Ok(Unsubscribe { pid, topics })
    }

    pub fn to_buffer(&self, buffer: &mut  BytesMut) -> Result<(), io::Error>{
        let header_u8 : u8 = 0b10100010;
        let PacketIdentifier(pid) = self.pid;
        let mut length = 2;
        for topic in &self.topics{
            length += 2 + topic.len();
        }

        buffer.put(header_u8);
        encoder::write_length(length, buffer)?;
        buffer.put_u16_be(pid as u16);
        for topic in&self.topics{
            encoder::write_string(topic.as_ref(), buffer)?;
        }
        Ok(())
    }
}

impl Suback {
    pub fn from_buffer(buffer: &mut BytesMut) -> Result<Self, io::Error> {
        let pid = PacketIdentifier(buffer.split_to(2).into_buf().get_u16_be());
        let mut return_codes: Vec<SubscribeReturnCodes> = Vec::new();
        while buffer.len() != 0 {
            let code = buffer.split_to(1).into_buf().get_u8();
            let r = if code == 0x80 {
                SubscribeReturnCodes::Failure
            } else {
                SubscribeReturnCodes::Success(QoS::from_u8(code)?)
            };
            return_codes.push(r);
        }
        Ok(Suback { return_codes, pid })
    }
    pub fn to_buffer(&self, buffer: &mut BytesMut) -> Result<(), io::Error> {
        let header_u8: u8 = 0b10010000;
        let PacketIdentifier(pid) = self.pid;
        let length = 2 + self.return_codes.len();

        buffer.put(header_u8);
        encoder::write_length(length, buffer)?;
        buffer.put_u16_be(pid);
        for rc in &self.return_codes {
            buffer.put(rc.to_u8());
        }
        Ok(())
    }
}
