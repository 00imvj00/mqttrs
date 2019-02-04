#[derive(Debug, Clone, PartialEq)]
pub enum PacketType {
    Connect,
    Connack,
    Publish,
    Puback,
    Pubrec,
    Pubrel,
    PubComp,
    Subscribe,
    SubAck,
    UnSubscribe,
    UnSubAck,
    PingReq,
    PingResp,
    Disconnect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    hd: u8,
    packet_type: PacketType,
    len: usize,
}
