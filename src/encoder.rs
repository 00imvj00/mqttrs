use crate::Packet;
use std::io;
use bytes::BytesMut;

pub fn encode(packet: &Packet, buffer: &mut BytesMut) -> Result<(), io::Error> {
    match packet {
        Packet::Connect(connect) => Ok(()), 
        Packet::Connack(connack) => Ok(()),
        Packet::Publish(publish) => Ok(()),
        Packet::Puback(pid) => Ok(()),
        Packet::Pubrec(pid) => Ok(()),
        Packet::Pubrel(pid) => Ok(()),
        Packet::PubComp(pid) => Ok(()),
        Packet::Subscribe(subscribe) => Ok(()),
        Packet::SubAck(suback) => Ok(()),
        Packet::UnSubscribe(unsub) => Ok(()),
        Packet::UnSubAck(pid) => Ok(()),
        Packet::PingReq => Ok(()),
        Packet::PingResp => Ok(()),
        Packet::Disconnect => Ok(()),
        Packet::None => Ok(()),
    }
}
