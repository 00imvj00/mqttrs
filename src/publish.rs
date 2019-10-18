use crate::{decoder::*, encoder::*, header::Header, Pid, QoS, QosPid};
use bytes::{BufMut, BytesMut};
use std::io::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Publish {
    pub dup: bool,
    pub qospid: QosPid,
    pub retain: bool,
    pub topic_name: String,
    pub payload: Vec<u8>,
}

impl Publish {
    pub(crate) fn from_buffer(header: &Header, buffer: &mut BytesMut) -> Result<Self, Error> {
        let topic_name = read_string(buffer)?;

        let qospid = match header.qos()? {
            QoS::AtMostOnce => QosPid::AtMostOnce,
            QoS::AtLeastOnce => QosPid::AtLeastOnce(Pid::from_buffer(buffer)?),
            QoS::ExactlyOnce => QosPid::ExactlyOnce(Pid::from_buffer(buffer)?),
        };

        let payload = buffer.to_vec();
        Ok(Publish {
            dup: header.dup(),
            qospid,
            retain: header.retain(),
            topic_name,
            payload,
        })
    }
    pub(crate) fn to_buffer(&self, buffer: &mut BytesMut) -> Result<(), Error> {
        // Header
        let mut header_u8: u8 = match self.qospid {
            QosPid::AtMostOnce => 0b00110000,
            QosPid::AtLeastOnce(_) => 0b00110010,
            QosPid::ExactlyOnce(_) => 0b00110100,
        };
        if self.dup {
            header_u8 |= 0b00001000 as u8;
        };
        if self.retain {
            header_u8 |= 0b00000001 as u8;
        };
        check_remaining(buffer, 1)?;
        buffer.put(header_u8);

        // Length: topic (2+len) + pid (0/2) + payload (len)
        let length = self.topic_name.len()
            + match self.qospid {
                QosPid::AtMostOnce => 2,
                _ => 4,
            }
            + self.payload.len();
        write_length(length, buffer)?;

        // Topic
        write_string(self.topic_name.as_ref(), buffer)?;

        // Pid
        match self.qospid {
            QosPid::AtMostOnce => (),
            QosPid::AtLeastOnce(pid) => pid.to_buffer(buffer)?,
            QosPid::ExactlyOnce(pid) => pid.to_buffer(buffer)?,
        }

        // Payload
        buffer.put_slice(self.payload.as_slice());

        Ok(())
    }
}
