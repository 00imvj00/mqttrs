mod connect;
mod decoder;
mod decoder_test;
mod encoder;
mod header;
mod packet;
mod publish;
mod subscribe;
mod utils;

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
