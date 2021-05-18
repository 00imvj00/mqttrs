use crate::{decoder::*, encoder::*, *};

/// Publish packet ([MQTT 3.3]).
///
/// [MQTT 3.3]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718037
#[derive(Debug, Clone, PartialEq)]
pub struct Publish<'a> {
    pub dup: bool,
    pub qospid: QosPid,
    pub retain: bool,
    pub topic_name: &'a str,
    pub payload: &'a [u8],
}

impl<'a> Publish<'a> {
    pub(crate) fn from_buffer(
        header: &Header,
        remaining_len: usize,
        buf: &'a [u8],
        offset: &mut usize,
    ) -> Result<Self, Error> {
        let payload_end = *offset + remaining_len;
        let topic_name = read_str(buf, offset)?;

        let qospid = match header.qos {
            QoS::AtMostOnce => QosPid::AtMostOnce,
            QoS::AtLeastOnce => QosPid::AtLeastOnce(Pid::from_buffer(buf, offset)?),
            QoS::ExactlyOnce => QosPid::ExactlyOnce(Pid::from_buffer(buf, offset)?),
        };

        Ok(Publish {
            dup: header.dup,
            qospid,
            retain: header.retain,
            topic_name,
            payload: &buf[*offset..payload_end],
        })
    }
    pub(crate) fn to_buffer(&self, buf: &mut [u8], offset: &mut usize) -> Result<usize, Error> {
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
        check_remaining(buf, offset, 1)?;
        write_u8(buf, offset, header)?;

        // Length: topic (2+len) + pid (0/2) + payload (len)
        let length = self.topic_name.len()
            + match self.qospid {
                QosPid::AtMostOnce => 2,
                _ => 4,
            }
            + self.payload.len();

        let write_len = write_length(buf, offset, length)? + 1;

        // Topic
        write_string(buf, offset, self.topic_name)?;

        // Pid
        match self.qospid {
            QosPid::AtMostOnce => (),
            QosPid::AtLeastOnce(pid) => pid.to_buffer(buf, offset)?,
            QosPid::ExactlyOnce(pid) => pid.to_buffer(buf, offset)?,
        }

        // Payload
        for &byte in self.payload {
            write_u8(buf, offset, byte)?;
        }

        Ok(write_len)
    }
}
