use crate::connect::Connect;
use crate::errors::Error;
use crate::header::read_header;
use crate::header::Header;
use crate::packet::{Packet, PacketType};
use std::convert::TryInto;

/// Decode bytes from a [BytesMut] buffer as a [Packet] enum.
///
/// The buf is never actually written to, it only takes a `BytesMut` instead of a `Bytes` to
/// allow using the same buffer to read bytes from network.
///
/// ```
/// # use mqttrs::*;
/// # use bytes::*;
/// // Fill a buffer with encoded data (probably from a `TcpStream`).
/// let mut buf = BytesMut::from(&[0b00110000, 11,
///                                0, 4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8,
///                                'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8] as &[u8]);
///
/// ```
///
/// [Packet]: ../enum.Packet.html
/// [BytesMut]: https://docs.rs/bytes/0.5.3/bytes/struct.BytesMut.html

/// Function to decode the mqtt packet coming from stream like TCP.
pub fn decode(data: &[u8]) -> Result<Option<(Packet, usize)>, Error> {
    let header: Header = data[0].try_into();

    let packet: (Packet, usize) = match header.typ {
        PacketType::Connect => {
            if let Some(readHeader) = read_header(&data)? {
                Some((
                    Packet::Connect(Connect::new(&data)),
                    readHeader.packet_length,
                ))
            }
        }
        _ => {}
        //PacketType::Connack => Packet::Connack(),
        //PacketType::Publish => Packet::Publish(),
        //PacketType::Puback => Packet::Puback(),
        //PacketType::Pubrec => Packet::Pubrec(),
        //PacketType::Pubrel => Packet::Pubrel(),
        //PacketType::Pubcomp => Packet::Pubcomp(),
        //PacketType::Subscribe => Packet::Subscribe(),
        //PacketType::Suback => Packet::Suback(),
        //PacketType::Unsubscribe => Packet::Unsubscribe(),
        //PacketType::Unsuback => Packet::Unsuback(),
        //PacketType::Pingreq => Packet::Pingreq,
        //PacketType::Pingresp => Packet::Pingresp,
        //PacketType::Disconnect => Packet::Disconnect,
    };

    Ok(Some(packet))
}
