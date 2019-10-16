use crate::{PacketType, QoS};
use bytes::{BufMut, BytesMut};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Header {
    hd: u8,
    packet_type: PacketType,
    len: usize,
}

impl Header {
    pub fn new(hd: u8, len: usize) -> Result<Header, io::Error> {
        Ok(Header {
            hd,
            len,
            packet_type: PacketType::from_hd(hd)?,
        })
    }
    pub fn packet(&self) -> PacketType {
        self.packet_type
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline]
    pub fn dup(&self) -> bool {
        (self.hd & 0b1000) != 0
    }
    #[inline]
    pub fn qos(&self) -> Result<QoS, io::Error> {
        QoS::from_hd(self.hd)
    }
    #[inline]
    pub fn retain(&self) -> bool {
        (self.hd & 1) != 0
    }
}

/// Protocol version sent at connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// [MQTT 3.1.1] is the most commonly implemented version. [MQTT 5] isn't yet supported my by
    /// `mqttrs`.
    ///
    /// [MQTT 3.1.1]: https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html
    /// [MQTT 5]: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html
    MQTT311,
    /// MQIsdp, aka SCADA are pre-standardisation names of MQTT. It should mostly conform to MQTT
    /// 3.1.1, but you should watch out for implementation discrepancies. `Mqttrs` handles it like
    /// standard MQTT 3.1.1.
    MQIsdp,
}
impl Protocol {
    pub(crate) fn new(name: &str, level: u8) -> Result<Protocol, io::Error> {
        match (name, level) {
            ("MQIsdp", 3) => Ok(Protocol::MQIsdp),
            ("MQTT", 4) => Ok(Protocol::MQTT311),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported protocol {:?} {}", name, level),
            )),
        }
    }
    pub(crate) fn to_buffer(&self, buffer: &mut BytesMut) -> Result<(), io::Error> {
        match self {
            Protocol::MQTT311 => {
                Ok(buffer.put_slice(&[0u8, 4, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 4]))
            }
            Protocol::MQIsdp => Ok(buffer.put_slice(&[
                0u8, 4, 'M' as u8, 'Q' as u8, 'i' as u8, 's' as u8, 'd' as u8, 'p' as u8, 4,
            ])),
        }
    }
}

/* TESTS */
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn header() {
        let h = Header::new(0b00010000, 0).unwrap();
        assert_eq!(h.packet(), PacketType::Connect)
    }
}
