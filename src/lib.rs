mod connect;
mod header;
mod packet;
mod publish;
mod subscribe;
mod utils;

#[cfg(test)]
mod codec_test;
#[cfg(test)]
mod decoder_test;
#[cfg(test)]
mod encoder_test;

pub mod decoder;
pub mod encoder;

pub use crate::{
    connect::{Connack, Connect},
    header::{Header, PacketType},
    packet::Packet,
    publish::Publish,
    subscribe::{Suback, Subscribe, SubscribeReturnCodes, SubscribeTopic, Unsubscribe},
    utils::{ConnectReturnCode, LastWill, PacketIdentifier, Protocol, QoS, QosPid},
};

const MULTIPLIER: usize = 0x80 * 0x80 * 0x80 * 0x80;
const MAX_PAYLOAD_SIZE: usize = 268435455;
