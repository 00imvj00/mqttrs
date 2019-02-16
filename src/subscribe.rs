use crate::{utils, PacketIdentifier, QoS};
use bytes::{Buf, BytesMut, IntoBuf};
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
}
