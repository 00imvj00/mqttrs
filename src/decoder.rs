use crate::{header::Header, *};
use bytes::{Buf, BytesMut, IntoBuf};
use std::io;

/// Decode network bytes into a [Packet] enum.
///
/// [Packet]: ../enum.Packet.html
pub fn decode(buffer: &mut BytesMut) -> Result<Option<Packet>, io::Error> {
    if let Some((header, header_size)) = read_header(buffer) {
        if buffer.len() >= header.len() + header_size {
            //NOTE: Check if buffer has, header bytes + remaining length bytes in buffer.
            buffer.split_to(header_size); //NOTE: Remove header bytes from buffer.
            let p = read_packet(header, buffer)?; //NOTE: Read remaining packet.
            Ok(Some(p))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn read_packet(header: Header, buffer: &mut BytesMut) -> Result<Packet, io::Error> {
    Ok(match header.packet() {
        PacketType::PingReq => Packet::PingReq,
        PacketType::PingResp => Packet::PingResp,
        PacketType::Disconnect => Packet::Disconnect,
        PacketType::Connect => {
            Packet::Connect(Connect::from_buffer(&mut buffer.split_to(header.len()))?)
        }
        PacketType::Connack => {
            Packet::Connack(Connack::from_buffer(&mut buffer.split_to(header.len()))?)
        }
        PacketType::Publish => Packet::Publish(Publish::from_buffer(
            &header,
            &mut buffer.split_to(header.len()),
        )?),
        PacketType::Puback => Packet::Puback(PacketIdentifier::from_buffer(buffer)?),
        PacketType::Pubrec => Packet::Pubrec(PacketIdentifier::from_buffer(buffer)?),
        PacketType::Pubrel => Packet::Pubrel(PacketIdentifier::from_buffer(buffer)?),
        PacketType::PubComp => Packet::PubComp(PacketIdentifier::from_buffer(buffer)?),
        PacketType::Subscribe => {
            Packet::Subscribe(Subscribe::from_buffer(&mut buffer.split_to(header.len()))?)
        }
        PacketType::SubAck => {
            Packet::SubAck(Suback::from_buffer(&mut buffer.split_to(header.len()))?)
        }
        PacketType::UnSubscribe => Packet::UnSubscribe(Unsubscribe::from_buffer(
            &mut buffer.split_to(header.len()),
        )?),
        PacketType::UnSubAck => Packet::UnSubAck(PacketIdentifier::from_buffer(buffer)?),
    })
}

/// Read the header of the stream
fn read_header(buffer: &mut BytesMut) -> Option<(Header, usize)> {
    if buffer.len() > 1 {
        let header_u8 = buffer.get(0).unwrap();
        if let Some((length, size)) = read_length(buffer, 1) {
            let header = Header::new(*header_u8, length).unwrap();
            Some((header, size + 1))
        } else {
            None
        }
    } else {
        None
    }
}

fn read_length(buffer: &BytesMut, mut pos: usize) -> Option<(usize, usize)> {
    let mut mult: usize = 1;
    let mut len: usize = 0;
    let mut done = false;

    while !done {
        let byte = (*buffer.get(pos).unwrap()) as usize;
        len += (byte & 0x7F) * mult;
        mult *= 0x80;
        if mult > MULTIPLIER {
            return None;
        }
        if (byte & 0x80) == 0 {
            done = true;
        } else {
            pos += 1;
        }
    }
    Some((len as usize, pos))
}

// FIXME: Result<String,...>
pub(crate) fn read_string(buffer: &mut BytesMut) -> String {
    String::from_utf8(read_bytes(buffer)).expect("Non-utf8 string")
}

// FIXME: This can panic if the packet is malformed
pub(crate) fn read_bytes(buffer: &mut BytesMut) -> Vec<u8> {
    let length = buffer.split_to(2).into_buf().get_u16_be();
    buffer.split_to(length as usize).to_vec()
}
