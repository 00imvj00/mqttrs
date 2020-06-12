//! `mqttrs` is a codec for the MQTT protocol.
//!
//! The API aims to be straightforward and composable, usable with plain `std` or with a framework
//! like [tokio]. The decoded packet is help in a [Packet] struct, and the encoded bytes in a
//! [bytes::BytesMut] struct. Convert between the two using [encode()] and [decode()]. Almost all
//! struct fields can be accessed directly, to create or read packets.
//!
//! It currently targets [MQTT 3.1], with [MQTT 5] support planned.
//!
//! ```
//! use mqttrs::*;
//! use bytes::BytesMut;
//!
//! // Allocate buffer.
//! let mut buf = BytesMut::with_capacity(1024);
//!
//! // Encode an MQTT Connect packet.
//! let pkt = Packet::Connect(Connect { protocol: Protocol::MQTT311,
//!                                     keep_alive: 30,
//!                                     client_id: "doc_client",
//!                                     clean_session: true,
//!                                     last_will: None,
//!                                     username: None,
//!                                     password: None });
//! assert!(encode(&pkt, &mut buf).is_ok());
//! assert_eq!(&buf[14..], b"doc_client");
//! let mut encoded = buf.clone();
//!
//! // Decode one packet. The buffer will advance to the next packet.
//! assert_eq!(Ok(Some(pkt)), decode_slice(&mut buf));
//!
//! // Example decode failures.
//! let mut incomplete = encoded.split_to(10);
//! assert_eq!(Ok(None), decode_slice(&mut incomplete));
//! let mut garbage = BytesMut::from(&[0u8,0,0,0] as &[u8]);
//! assert_eq!(Err(Error::InvalidHeader), decode_slice(&mut garbage));
//! ```
//!
//! [MQTT 3.1]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html
//! [MQTT 5]: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html
//! [tokio]: https://tokio.rs/
//! [Packet]: enum.Packet.html
//! [encode()]: fn.encode.html
//! [decode_slice()]: fn.decode_slice.html
//! [bytes::BytesMut]: https://docs.rs/bytes/0.5.3/bytes/struct.BytesMut.html

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "std")]
extern crate std;

mod connect;
mod decoder;
mod encoder;
mod packet;
mod publish;
mod subscribe;
mod utils;

// Proptest does not currently support borrowed data in strategies:
// https://github.com/AltSysrq/proptest/issues/9
//
// #[cfg(test)]
// mod codec_test;
#[cfg(test)]
mod decoder_test;
#[cfg(test)]
mod encoder_test;

pub use crate::{
    connect::{Connack, Connect, ConnectReturnCode, LastWill, Protocol},
    decoder::{clone_packet, decode_slice},
    encoder::encode,
    packet::{Packet, PacketType},
    publish::Publish,
    subscribe::{Suback, Subscribe, SubscribeReturnCodes, SubscribeTopic, Unsubscribe},
    utils::{Error, Pid, QoS, QosPid},
};
