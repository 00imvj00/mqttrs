
use crate::{QoS, PacketIdentifier};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct Publish {
    pub dup: bool,
    pub qos: QoS,
    pub retain: bool,
    pub topic_name: String,
    pub pid: Option<PacketIdentifier>,
    pub payload: Arc<Vec<u8>>
}