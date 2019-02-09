use crate::{Connack, Connect, PacketIdentifier, Publish, Suback, Subscribe, Unsubscribe};

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    Connect(Connect),
    Connack(Connack),
    Publish(Publish),
    Puback(PacketIdentifier),
    Pubrec(PacketIdentifier),
    Pubrel(PacketIdentifier),
    PubComp(PacketIdentifier),
    Subscribe(Subscribe),
    SubAck(Suback),
    UnSubscribe(Unsubscribe),
    UnSubAck(PacketIdentifier),
    PingReq,
    PingResp,
    Disconnect,
}
