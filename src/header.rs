use crate::{Error, PacketType, QoS};
use bytes::{BufMut, BytesMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Header {
    pub typ: PacketType,
    pub dup: bool,
    pub qos: QoS,
    pub retain: bool,
}

impl Header {
    pub fn new(hd: u8) -> Result<Header, Error> {
        let (typ, flags_ok) = match hd >> 4 {
            1 => (PacketType::Connect, hd & 0b1111 == 0),
            2 => (PacketType::Connack, hd & 0b1111 == 0),
            3 => (PacketType::Publish, true),
            4 => (PacketType::Puback, hd & 0b1111 == 0),
            5 => (PacketType::Pubrec, hd & 0b1111 == 0),
            6 => (PacketType::Pubrel, hd & 0b1111 == 0b0010),
            7 => (PacketType::Pubcomp, hd & 0b1111 == 0),
            8 => (PacketType::Subscribe, hd & 0b1111 == 0b0010),
            9 => (PacketType::Suback, hd & 0b1111 == 0),
            10 => (PacketType::Unsubscribe, hd & 0b1111 == 0b0010),
            11 => (PacketType::Unsuback, hd & 0b1111 == 0),
            12 => (PacketType::Pingreq, hd & 0b1111 == 0),
            13 => (PacketType::Pingresp, hd & 0b1111 == 0),
            14 => (PacketType::Disconnect, hd & 0b1111 == 0),
            _ => (PacketType::Connect, false),
        };
        if !flags_ok {
            return Err(Error::InvalidHeader);
        }
        Ok(Header {
            typ,
            dup: hd & 0b1000 != 0,
            qos: QoS::from_u8((hd & 0b110) >> 1)?,
            retain: hd & 1 == 1,
        })
    }
}

/// Protocol version.
///
/// Sent in [`Connect`] packet.
///
/// [`Connect`]: struct.Connect.html
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
    pub(crate) fn new(name: &str, level: u8) -> Result<Protocol, Error> {
        match (name, level) {
            ("MQIsdp", 3) => Ok(Protocol::MQIsdp),
            ("MQTT", 4) => Ok(Protocol::MQTT311),
            _ => Err(Error::InvalidProtocol(name.into(), level)),
        }
    }
    pub(crate) fn to_buffer(&self, buffer: &mut BytesMut) -> Result<(), Error> {
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
