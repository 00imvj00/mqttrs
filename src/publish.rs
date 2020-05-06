use crate::{decoder::*, encoder::*, *};
use alloc::{string::String, vec::Vec};
use bytes::{Buf, BufMut};

/// Publish packet ([MQTT 3.3]).
///
/// [MQTT 3.3]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718037
#[derive(Debug, Clone, PartialEq)]
pub struct Publish {
    pub dup: bool,
    pub qospid: QosPid,
    pub retain: bool,
    pub topic_name: String,
    pub payload: Vec<u8>,
}

impl Publish {
    pub(crate) fn from_buffer(header: &Header, mut buf: impl Buf) -> Result<Self, Error> {
        let topic_name = read_string(&mut buf)?;

        let qospid = match header.qos {
            QoS::AtMostOnce => QosPid::AtMostOnce,
            QoS::AtLeastOnce => QosPid::AtLeastOnce(Pid::from_buffer(&mut buf)?),
            QoS::ExactlyOnce => QosPid::ExactlyOnce(Pid::from_buffer(&mut buf)?),
        };

        Ok(Publish {
            dup: header.dup,
            qospid,
            retain: header.retain,
            topic_name,
            payload: buf.bytes().to_vec(),
        })
    }
    pub(crate) fn to_buffer(&self, mut buf: impl BufMut) -> Result<usize, Error> {
        // Header
        let mut header: u8 = match self.qospid {
            QosPid::AtMostOnce => 0b00110000,
            QosPid::AtLeastOnce(_) => 0b00110010,
            QosPid::ExactlyOnce(_) => 0b00110100,
        };
        if self.dup {
            header |= 0b00001000 as u8;
        };
        if self.retain {
            header |= 0b00000001 as u8;
        };
        check_remaining(&mut buf, 1)?;
        buf.put_u8(header);

        // Length: topic (2+len) + pid (0/2) + payload (len)
        let length = self.topic_name.len()
            + match self.qospid {
                QosPid::AtMostOnce => 2,
                _ => 4,
            }
            + self.payload.len();

        let write_len = write_length(length, &mut buf)? + 1;

        // Topic
        write_string(self.topic_name.as_ref(), &mut buf)?;

        // Pid
        match self.qospid {
            QosPid::AtMostOnce => (),
            QosPid::AtLeastOnce(pid) => pid.to_buffer(&mut buf)?,
            QosPid::ExactlyOnce(pid) => pid.to_buffer(&mut buf)?,
        }

        // Payload
        buf.put_slice(self.payload.as_slice());

        Ok(write_len)
    }
}
