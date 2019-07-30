use crate::{encoder, utils, Header, PacketIdentifier, QoS};
use bytes::{Buf, BufMut, BytesMut, IntoBuf};
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
    pub fn from_buffer(header: &Header, buffer: &mut BytesMut) -> Result<Self, io::Error> {
        let topic_name = utils::read_string(buffer);

        let pid = if header.qos()? == QoS::AtMostOnce {
            None
        } else {
            Some(PacketIdentifier(buffer.split_to(2).into_buf().get_u16_be()))
        };
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
    pub fn to_buffer(&self, buffer: &mut BytesMut) -> Result<(), io::Error> {
        let mut header_u8: u8 = 0b00110000 as u8;
        let mut length = 0;
        header_u8 |= (self.qos.to_u8()) << 1;
        if self.dup {
            header_u8 |= 0b00001000 as u8;
        };
        if self.retain {
            header_u8 |= 0b00000001 as u8;
        };

        let PacketIdentifier(pid) = self.pid.unwrap();
        length = length + 2 + self.topic_name.len() + 2;
        length += self.payload.len();
        buffer.put(header_u8);
        encoder::write_length(length, buffer)?;
        encoder::write_string(self.topic_name.as_ref(), buffer)?;
        buffer.put_u16_be(pid as u16);
        buffer.put_slice(self.payload.as_slice());

        Ok(())
    }
}
