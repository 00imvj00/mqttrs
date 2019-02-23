use crate::MULTIPLIER;
use crate::*;
use bytes::{Buf, BytesMut, IntoBuf};
use std::io;

#[allow(dead_code)]
pub fn decode(buffer: &mut BytesMut) -> Result<Option<Packet>, io::Error> {
    if let Some((header, header_size)) = read_header(buffer) {
        buffer.split_to(header_size);
        if buffer.len() >= header.len() {
            let p = read_packet(header, buffer)?;
            Ok(Some(p))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn read_packet(header: Header, buffer: &mut BytesMut) -> Result<Packet, io::Error> {
    let t = header.packet();
    match t {
        PacketType::PingReq => Ok(Packet::PingReq),
        PacketType::PingResp => Ok(Packet::PingResp),
        PacketType::Disconnect => Ok(Packet::Disconnect),
        PacketType::Connect => Ok(Packet::Connect(Connect::from_buffer(
            &mut buffer.split_to(header.len()),
        )?)),
        PacketType::Connack => Ok(Packet::Connack(Connack::from_buffer(
            &mut buffer.split_to(header.len()),
        )?)),
        PacketType::Publish => Ok(Packet::Publish(Publish::from_buffer(
            &header,
            &mut buffer.split_to(header.len()),
        )?)),
        PacketType::Puback => Ok(Packet::Puback(PacketIdentifier(
            buffer.split_to(2).into_buf().get_u16_be(),
        ))),
        PacketType::Pubrec => Ok(Packet::Pubrec(PacketIdentifier(
            buffer.split_to(2).into_buf().get_u16_be(),
        ))),
        PacketType::Pubrel => Ok(Packet::Pubrel(PacketIdentifier(
            buffer.split_to(2).into_buf().get_u16_be(),
        ))),
        PacketType::PubComp => Ok(Packet::PubComp(PacketIdentifier(
            buffer.split_to(2).into_buf().get_u16_be(),
        ))),
        PacketType::Subscribe => Ok(Packet::Subscribe(Subscribe::from_buffer(
            &mut buffer.split_to(header.len()),
        )?)),
        PacketType::SubAck => Ok(Packet::SubAck(Suback::from_buffer(
            &mut buffer.split_to(header.len()),
        )?)),
        PacketType::UnSubscribe => Ok(Packet::UnSubscribe(Unsubscribe::from_buffer(
            &mut buffer.split_to(header.len()),
        )?)),
        PacketType::UnSubAck => Ok(Packet::UnSubAck(PacketIdentifier(
            buffer.split_to(2).into_buf().get_u16_be(),
        ))),
    }
}
/* This will read the header of the stream */
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
