#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PacketIdentifier(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    MQIsdp(u8),
    MQTT(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectReturnCode {
    Accepted,
    RefusedProtocolVersion,
    RefusedIdentifierRejected,
    ServerUnavailable,
    BadUsernamePassword,
    NotAuthorized,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LastWill {
    pub topic: String,
    pub message: String,
    pub qos: QoS,
    pub retain: bool,
}