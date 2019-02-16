use crate::{utils, Header, PacketIdentifier, QoS};
use bytes::{Buf, BytesMut, IntoBuf};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct Publish {
    pub dup: bool,
    pub qos: QoS,
    pub retain: bool,
    pub topic_name: String,
    pub pid: Option<PacketIdentifier>,
    pub payload: Vec<u8>,
}

impl Publish {
    pub fn from_buffer(header: Header, buffer: &mut BytesMut) -> Result<Self, io::Error> {
        let topic_name = utils::read_string(buffer);
        let pid = Some(PacketIdentifier(buffer.split_to(2).into_buf().get_u16_be()));
        let payload = buffer.to_vec();
        Ok(Publish {
            dup: header.dup(),
            qos: header.qos()?,
            retain: header.retain(),
            topic_name,
            pid,
            payload,
        })
    }
}
