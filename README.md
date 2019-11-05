# Rust Mqtt Encoding & Decoding  [![Crates.io](https://img.shields.io/crates/l/mqttrs)](LICENSE) [![Docs.rs](https://docs.rs/mqttrs/badge.svg)](https://docs.rs/mqttrs/*/mqttrs/)

`Mqttrs` is a [Rust](https://www.rust-lang.org/) crate (library) to write [MQTT
protocol](https://mqtt.org/) clients and servers.

It is a codec-only library with [very few dependencies](Cargo.toml) and a [straightworward and
composable API](https://docs.rs/mqttrs/*/mqttrs/), usable with rust's standard library or with async
frameworks like [tokio](https://tokio.rs/). It is strict when decoding (e.g. returns an error when
encountering reserved values) and encoding (the API makes it impossible to generate an illegal
packet).

`Mqttrs` currently requires [Rust >= 1.32](https://www.rust-lang.org/learn/get-started) and supports
[MQTT 3.1.1](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html). Support for [MQTT
5](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html) is planned for a future version.


## Usage

Add `mqttrs = "0.2"` to your `Cargo.toml`.

```rust
use mqttrs::*;
use bytes::BytesMut;

// Allocate write buffer.
let mut buf = BytesMut::with_capacity(1024);

// Encode an MQTT Connect packet.
let pkt = Packet::Connect(Connect { protocol: Protocol::MQTT311,
                                    keep_alive: 30,
                                    client_id: "doc_client".into(),
                                    clean_session: true,
                                    last_will: None,
                                    username: None,
                                    password: None });
assert!(encode(&pkt, &mut buf).is_ok());
assert_eq!(&buf[14..], "doc_client".as_bytes());
let mut encoded = buf.clone();

// Decode one packet. The buffer will advance to the next packet.
assert_eq!(Ok(Some(pkt)), decode(&mut buf));

// Example decode failures.
let mut incomplete = encoded.split_to(10);
assert_eq!(Ok(None), decode(&mut incomplete));
let mut garbage = BytesMut::from(vec![0u8,0,0,0]);
assert_eq!(Err(Error::InvalidHeader), decode(&mut garbage));
```

## Optional [serde](https://serde.rs/) support.

Use  `mqttrs = { version = "0.2", features = [ "derive" ] }` in your `Cargo.toml`.

Enabling this features adds `#[derive(Deserialize, Serialize)]` to some `mqttrs` types. This
simplifies storing those structs in a database or file, typically to implement session support (qos,
subscriptions...).

This doesn't add mqtt as a serde data format; you still need to use the `mqttrs::{decode,encode}`
functions.
