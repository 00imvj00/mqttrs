use crate::Packet;
use std::io;
use bytes::{Buf,BytesMut, BufMut};

pub fn encode(packet: &Packet, buffer: &mut BytesMut) -> Result<(), io::Error> {
    match packet {
        Packet::Connect(connect) => Ok(()), 
        Packet::Connack(connack) => Ok(()),
        Packet::Publish(publish) => Ok(()),
        Packet::Puback(pid) => {
            let header_u8 = 0b01000000 as u8; 
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            buffer.put_u16_be(pid.0);
            Ok(())
        },
        Packet::Pubrec(pid) => {
            let header_u8 = 0b01010000 as u8; 
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            buffer.put_u16_be(pid.0);
            Ok(())
        },
        Packet::Pubrel(pid) => {
            let header_u8 = 0b01100000 as u8; 
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            buffer.put_u16_be(pid.0);
            Ok(())
        },
        Packet::PubComp(pid) => {
            let header_u8 = 0b01110000 as u8; 
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            buffer.put_u16_be(pid.0);
            Ok(())
        },
        Packet::Subscribe(subscribe) => Ok(()),
        Packet::SubAck(suback) => Ok(()),
        Packet::UnSubscribe(unsub) => Ok(()),
        Packet::UnSubAck(pid) => {
            let header_u8 = 0b10110000 as u8; 
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            buffer.put_u16_be(pid.0);
            Ok(())
        },
        Packet::PingReq => {
            buffer.put(0b11000000 as u8);
            Ok(())
        },
        Packet::PingResp => {
            buffer.put(0b11010000 as u8);
            Ok(())
        },
        Packet::Disconnect => {
            buffer.put(0b11100000 as u8);
            Ok(())
        },
        Packet::None => Ok(()),
    }
}
