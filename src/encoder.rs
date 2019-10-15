use crate::{Packet, MAX_PAYLOAD_SIZE};
use bytes::{BufMut, BytesMut};
use std::io;

#[allow(dead_code)]
pub fn encode(packet: &Packet, buffer: &mut BytesMut) -> Result<(), io::Error> {
    match packet {
        Packet::Connect(connect) => connect.to_buffer(buffer),
        Packet::Connack(connack) => connack.to_buffer(buffer),
        Packet::Publish(publish) => publish.to_buffer(buffer),
        Packet::Puback(pid) => {
            let header_u8 = 0b01000000 as u8;
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            buffer.put_u16_be(pid.get());
            Ok(())
        }
        Packet::Pubrec(pid) => {
            let header_u8 = 0b01010000 as u8;
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            buffer.put_u16_be(pid.get());
            Ok(())
        }
        Packet::Pubrel(pid) => {
            let header_u8 = 0b01100000 as u8;
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            buffer.put_u16_be(pid.get());
            Ok(())
        }
        Packet::PubComp(pid) => {
            let header_u8 = 0b01110000 as u8;
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            buffer.put_u16_be(pid.get());
            Ok(())
        }
        Packet::Subscribe(subscribe) => subscribe.to_buffer(buffer),
        Packet::SubAck(suback) => suback.to_buffer(buffer),
        Packet::UnSubscribe(unsub) => unsub.to_buffer(buffer),
        Packet::UnSubAck(pid) => {
            let header_u8 = 0b10110000 as u8;
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            buffer.put_u16_be(pid.get());
            Ok(())
        }
        Packet::PingReq => {
            buffer.put(0b11000000 as u8);
            buffer.put(0b00000000 as u8);
            Ok(())
        }
        Packet::PingResp => {
            buffer.put(0b11010000 as u8);
            buffer.put(0b00000000 as u8);
            Ok(())
        }
        Packet::Disconnect => {
            buffer.put(0b11100000 as u8);
            buffer.put(0b00000000 as u8);
            Ok(())
        }
    }
}

pub fn write_length(len: usize, buffer: &mut BytesMut) -> Result<(), io::Error> {
    if len > MAX_PAYLOAD_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "data size too big",
        ));
    };
    let mut done = false;
    let mut x = len;
    while !done {
        let mut byte = (x % 128) as u8;
        x = x / 128;
        if x > 0 {
            byte = byte | 128;
        }
        buffer.put(byte as u8);
        done = x <= 0;
    }
    Ok(())
}

pub fn write_string(string: &str, buffer: &mut BytesMut) -> Result<(), io::Error> {
    buffer.put_u16_be(string.len() as u16);
    buffer.put_slice(string.as_bytes());
    Ok(())
}
