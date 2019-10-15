use crate::{encoder, utils, Header, PacketIdentifier, QoS};
use bytes::{BufMut, BytesMut};
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
            Some(PacketIdentifier::from_buffer(buffer)?)
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
        // Header
        let mut header_u8: u8 = 0b00110000 as u8;
        header_u8 |= (self.qos.to_u8()) << 1;
        if self.dup {
            header_u8 |= 0b00001000 as u8;
        };
        if self.retain {
            header_u8 |= 0b00000001 as u8;
        };
        buffer.put(header_u8);

        // Length: topic (2+len) + pid (0/2) + payload (len)
        let length = self.topic_name.len()
            + match self.qos {
                QoS::AtMostOnce => 2,
                _ => 4,
            }
            + self.payload.len();
        encoder::write_length(length, buffer)?;

        // Topic
        encoder::write_string(self.topic_name.as_ref(), buffer)?;

        // Pid
        if self.qos != QoS::AtMostOnce {
            self.pid.unwrap().to_buffer(buffer);
        }

        // Payload
        buffer.put_slice(self.payload.as_slice());

        Ok(())
    }
}
