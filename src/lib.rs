mod connect;
mod decoder_test;
mod encoder_test;
mod header;
mod packet;
mod publish;
mod subscribe;
mod utils;

pub mod decoder;
pub mod encoder;

pub use crate::{
    connect::{Connack, Connect},
    header::{Header, PacketType},
    packet::Packet,
    publish::Publish,
    subscribe::{Suback, Subscribe, SubscribeReturnCodes, SubscribeTopic, Unsubscribe},
    utils::{ConnectReturnCode, LastWill, PacketIdentifier, Protocol, QoS},
};

const MULTIPLIER: usize = 0x80 * 0x80 * 0x80 * 0x80;
const MAX_PAYLOAD_SIZE: usize = 268435455;
