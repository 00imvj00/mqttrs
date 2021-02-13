use crate::{errors::Error, packet::PacketType, qos::QoS};
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub typ: PacketType,
    pub dup: bool,
    pub qos: QoS,
    pub retain: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReadHeader {
    pub header: Header,
    pub remaining_length: usize,
    pub packet_length: usize,
}

impl TryFrom<u8> for Header {
    type Error = Error;
    fn try_from(hd: u8) -> Result<Self, Self::Error> {
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

pub(crate) fn read_header(data: &[u8]) -> Result<Option<ReadHeader>, Error> {
    let mut len: usize = 0; /* future remaining length*/

    /*
         The length of the remaining length field is between 1 and 4 bytes
         depending on the payload size (the actual user message).

         Which means, we have to process from data[1] to data[4] bytes.

         Here, We take first
    */
    for pos in 1..5 {
        match data[pos] {
            byte => {
                len += (byte & 0b01111111) << ((pos - 1) * 7);
                /*check MSB === 1, to know if there is more length to add or not*/
                if (byte & 0b1000000) == 0 {
                    let total = 1 + pos + len;
                    if data.len() < total {
                        return Ok(None);
                    }
                    let header = Header::try_from(data[0])?;
                    return Ok(ReadHeader {
                        header,
                        remaining_length: len,
                        packet_length: total,
                    });
                }
            }
            _ => {
                /* We didn't receive all the bytes yet. */
                return Ok(None);
            }
        }
    }

    Err(Error::InvalidHeader)
}
