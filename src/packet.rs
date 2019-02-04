#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
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
