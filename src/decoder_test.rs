use crate::*;
use bytes::BytesMut;

macro_rules! header {
    ($t:ident, $d:expr, $q:ident, $r:expr) => {
        decoder::Header {
            typ: PacketType::$t,
            dup: $d,
            qos: QoS::$q,
            retain: $r,
        }
    };
}

fn bm(d: &[u8]) -> BytesMut {
    BytesMut::from(d)
}

/// Test all possible header first byte, using remaining_len=0.
#[test]
fn header_firstbyte() {
    let valid = vec![
        (0b0001_0000, header!(Connect, false, AtMostOnce, false)),
        (0b0010_0000, header!(Connack, false, AtMostOnce, false)),
        (0b0011_0000, header!(Publish, false, AtMostOnce, false)),
        (0b0011_0001, header!(Publish, false, AtMostOnce, true)),
        (0b0011_0010, header!(Publish, false, AtLeastOnce, false)),
        (0b0011_0011, header!(Publish, false, AtLeastOnce, true)),
        (0b0011_0100, header!(Publish, false, ExactlyOnce, false)),
        (0b0011_0101, header!(Publish, false, ExactlyOnce, true)),
        (0b0011_1000, header!(Publish, true, AtMostOnce, false)),
        (0b0011_1001, header!(Publish, true, AtMostOnce, true)),
        (0b0011_1010, header!(Publish, true, AtLeastOnce, false)),
        (0b0011_1011, header!(Publish, true, AtLeastOnce, true)),
        (0b0011_1100, header!(Publish, true, ExactlyOnce, false)),
        (0b0011_1101, header!(Publish, true, ExactlyOnce, true)),
        (0b0100_0000, header!(Puback, false, AtMostOnce, false)),
        (0b0101_0000, header!(Pubrec, false, AtMostOnce, false)),
        (0b0110_0010, header!(Pubrel, false, AtLeastOnce, false)),
        (0b0111_0000, header!(Pubcomp, false, AtMostOnce, false)),
        (0b1000_0010, header!(Subscribe, false, AtLeastOnce, false)),
        (0b1001_0000, header!(Suback, false, AtMostOnce, false)),
        (0b1010_0010, header!(Unsubscribe, false, AtLeastOnce, false)),
        (0b1011_0000, header!(Unsuback, false, AtMostOnce, false)),
        (0b1100_0000, header!(Pingreq, false, AtMostOnce, false)),
        (0b1101_0000, header!(Pingresp, false, AtMostOnce, false)),
        (0b1110_0000, header!(Disconnect, false, AtMostOnce, false)),
    ];
    for n in 0..=255 {
        let res = match valid.iter().find(|(byte, _)| *byte == n) {
            Some((_, header)) => Ok(Some((*header, 0))),
            None if ((n & 0b110) == 0b110) && (n >> 4 == 3) => Err(Error::InvalidQos(3)),
            None => Err(Error::InvalidHeader),
        };
        let mut buf: &[u8] = &[n, 0];
        let mut offset = 0;
        assert_eq!(
            res,
            decoder::read_header(&mut buf, &mut offset),
            "{:08b}",
            n
        );
        if res.is_ok() {
            assert_eq!(offset, 2);
        } else {
            assert_eq!(offset, 0);
        }
    }
}

/// Test decoding of length and actual buffer len.
#[rustfmt::skip]
#[test]
fn header_len() {
    let h = header!(Connect, false, AtMostOnce, false);
    for (res, mut bytes, buflen) in vec![
        (Ok(Some((h, 0))),          vec![1 << 4, 0],   2),
        (Ok(None),                  vec![1 << 4, 127], 128),
        (Ok(Some((h, 127))),        vec![1 << 4, 127], 129),
        (Ok(None),                  vec![1 << 4, 0x80], 2),
        (Ok(Some((h, 0))),          vec![1 << 4, 0x80, 0], 3), //Weird encoding for "0" buf matches spec
        (Ok(Some((h, 128))),        vec![1 << 4, 0x80, 1], 131),
        (Ok(None),                  vec![1 << 4, 0x80+16, 78], 10002),
        (Ok(Some((h, 10000))),      vec![1 << 4, 0x80+16, 78], 10003),
        (Err(Error::InvalidHeader), vec![1 << 4, 0x80, 0x80, 0x80, 0x80], 10),
    ] {
        let offset_expectation = bytes.len();
        bytes.resize(buflen, 0);
        let mut slice_buf = bytes.as_slice();
        let mut offset = 0;
        assert_eq!(res, decoder::read_header(&mut slice_buf, &mut offset));
        match res {
            Ok(Some(_)) => assert_eq!(offset, offset_expectation),
            _ => assert_eq!(offset, 0)
        }
    }
}

#[test]
fn non_utf8_string() {
    let mut data: &[u8] = &[
        0b00110000, 10, // type=Publish, remaining_len=10
        0x00, 0x03, 'a' as u8, '/' as u8, 0xc0 as u8, // Topic with Invalid utf8
        'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, // payload
    ];
    assert!(match decode_slice(&mut data) {
        Err(Error::InvalidString(_)) => true,
        _ => false,
    });
}

/// Validity of remaining_len is tested exhaustively elsewhere, this is for inner lengths, which
/// are rarer.
#[test]
fn inner_length_too_long() {
    let mut data = bm(&[
        0b00010000, 20, // Connect packet, remaining_len=20
        0x00, 0x04, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 0x04, 0b01000000, // +password
        0x00, 0x0a, // keepalive 10 sec
        0x00, 0x04, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, // client_id
        0x00, 0x03, 'm' as u8, 'q' as u8, // password with invalid length
    ]);
    assert_eq!(Err(Error::InvalidLength), decode_slice(&mut data));

    let mut slice: &[u8] = &[
        0b00010000, 20, // Connect packet, remaining_len=20
        0x00, 0x04, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 0x04, 0b01000000, // +password
        0x00, 0x0a, // keepalive 10 sec
        0x00, 0x04, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, // client_id
        0x00, 0x03, 'm' as u8, 'q' as u8, // password with invalid length
    ];

    assert_eq!(Err(Error::InvalidLength), decode_slice(&mut slice));
    // assert_eq!(slice, []);
}

#[test]
fn test_half_connect() {
    let mut data: &[u8] = &[
        0b00010000, 39, 0x00, 0x04, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 0x04,
        0b11001110, // +username, +password, -will retain, will qos=1, +last_will, +clean_session
        0x00,
        0x0a, // 10 sec
              // 0x00, 0x04, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, // client_id
              // 0x00, 0x02, '/' as u8, 'a' as u8, // will topic = '/a'
              // 0x00, 0x07, 'o' as u8, 'f' as u8, 'f' as u8, 'l' as u8, 'i' as u8, 'n' as u8,
              // 'e' as u8, // will msg = 'offline'
              // 0x00, 0x04, 'r' as u8, 'u' as u8, 's' as u8, 't' as u8, // username = 'rust'
              // 0x00, 0x02, 'm' as u8, 'q' as u8, // password = 'mq'
    ];
    assert_eq!(Ok(None), decode_slice(&mut data));
    assert_eq!(12, data.len());
}

#[test]
fn test_connect() {
    let mut data: &[u8] = &[
        0b00010000, 39, 0x00, 0x04, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 0x04,
        0b11001110, // +username, +password, -will retain, will qos=1, +last_will, +clean_session
        0x00, 0x0a, // 10 sec
        0x00, 0x04, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, // client_id
        0x00, 0x02, '/' as u8, 'a' as u8, // will topic = '/a'
        0x00, 0x07, 'o' as u8, 'f' as u8, 'f' as u8, 'l' as u8, 'i' as u8, 'n' as u8,
        'e' as u8, // will msg = 'offline'
        0x00, 0x04, 'r' as u8, 'u' as u8, 's' as u8, 't' as u8, // username = 'rust'
        0x00, 0x02, 'm' as u8, 'q' as u8, // password = 'mq'
    ];
    let pkt = Connect {
        protocol: Protocol::MQTT311,
        keep_alive: 10,
        client_id: "test",
        clean_session: true,
        last_will: Some(LastWill {
            topic: "/a",
            message: b"offline",
            qos: QoS::AtLeastOnce,
            retain: false,
        }),
        username: Some("rust"),
        password: Some(b"mq"),
    };

    let packet_buf = &mut [0u8; 64];
    assert_eq!(
        clone_packet(&mut data, &mut packet_buf[..]).unwrap(),
        Some(41)
    );
    assert_eq!(Ok(Some(pkt.into())), decode_slice(packet_buf));
    assert_eq!(data.len(), 0);
}

#[test]
fn test_connack() {
    let mut data: &[u8] = &[0b00100000, 2, 0b00000000, 0b00000001];
    let d = decode_slice(&mut data).unwrap();
    match d {
        Some(Packet::Connack(c)) => {
            let o = Connack {
                session_present: false,
                code: ConnectReturnCode::RefusedProtocolVersion,
            };
            assert_eq!(c.session_present, o.session_present);
            assert_eq!(c.code, o.code);
        }
        _ => panic!(),
    }
}

#[test]
fn test_ping_req() {
    let mut data: &[u8] = &[0b11000000, 0b00000000];
    assert_eq!(Ok(Some(Packet::Pingreq)), decode_slice(&mut data));
}

#[test]
fn test_ping_resp() {
    let mut data: &[u8] = &[0b11010000, 0b00000000];
    assert_eq!(Ok(Some(Packet::Pingresp)), decode_slice(&mut data));
}

#[test]
fn test_disconnect() {
    let mut data: &[u8] = &[0b11100000, 0b00000000];
    assert_eq!(Ok(Some(Packet::Disconnect)), decode_slice(&mut data));
}

#[test]
fn test_publish() {
    let mut data: &[u8] = &[
        0b00110000, 10, 0x00, 0x03, 'a' as u8, '/' as u8, 'b' as u8, 'h' as u8, 'e' as u8,
        'l' as u8, 'l' as u8, 'o' as u8, //
        0b00111000, 10, 0x00, 0x03, 'a' as u8, '/' as u8, 'b' as u8, 'h' as u8, 'e' as u8,
        'l' as u8, 'l' as u8, 'o' as u8, //
        0b00111101, 12, 0x00, 0x03, 'a' as u8, '/' as u8, 'b' as u8, 0, 10, 'h' as u8, 'e' as u8,
        'l' as u8, 'l' as u8, 'o' as u8,
    ];

    let mut offset = 0;
    assert_eq!(
        decoder::read_header(&data, &mut offset).unwrap(),
        Some((decoder::Header::new(0b00110000).unwrap(), 10))
    );
    assert_eq!(data.len(), 38);

    let packet_buf = &mut [0u8; 64];
    assert_eq!(
        clone_packet(&mut data, &mut packet_buf[..]).unwrap(),
        Some(12)
    );
    assert_eq!(data.len(), 26);

    match decode_slice(packet_buf) {
        Ok(Some(Packet::Publish(p))) => {
            assert_eq!(p.dup, false);
            assert_eq!(p.retain, false);
            assert_eq!(p.qospid, QosPid::AtMostOnce);
            assert_eq!(p.topic_name, "a/b");
            assert_eq!(core::str::from_utf8(p.payload).unwrap(), "hello");
        }
        other => panic!("Failed decode: {:?}", other),
    }

    let packet_buf2 = &mut [0u8; 64];
    assert_eq!(
        clone_packet(&mut data, &mut packet_buf2[..]).unwrap(),
        Some(12)
    );
    assert_eq!(data.len(), 14);
    match decode_slice(packet_buf2) {
        Ok(Some(Packet::Publish(p))) => {
            assert_eq!(p.dup, true);
            assert_eq!(p.retain, false);
            assert_eq!(p.qospid, QosPid::AtMostOnce);
            assert_eq!(p.topic_name, "a/b");
            assert_eq!(core::str::from_utf8(p.payload).unwrap(), "hello");
        }
        other => panic!("Failed decode: {:?}", other),
    }

    let packet_buf3 = &mut [0u8; 64];
    assert_eq!(
        clone_packet(&mut data, &mut packet_buf3[..]).unwrap(),
        Some(14)
    );
    assert_eq!(data.len(), 0);

    match decode_slice(packet_buf3) {
        Ok(Some(Packet::Publish(p))) => {
            assert_eq!(p.dup, true);
            assert_eq!(p.retain, true);
            assert_eq!(p.qospid, QosPid::from_u8u16(2, 10));
            assert_eq!(p.topic_name, "a/b");
            assert_eq!(core::str::from_utf8(p.payload).unwrap(), "hello");
        }
        other => panic!("Failed decode: {:?}", other),
    }
}

#[test]
fn test_pub_ack() {
    let mut data: &[u8] = &[0b01000000, 0b00000010, 0, 10];
    match decode_slice(&mut data) {
        Ok(Some(Packet::Puback(a))) => assert_eq!(a.get(), 10),
        other => panic!("Failed decode: {:?}", other),
    };
}

#[test]
fn test_pub_rec() {
    let mut data: &[u8] = &[0b01010000, 0b00000010, 0, 10];
    match decode_slice(&mut data) {
        Ok(Some(Packet::Pubrec(a))) => assert_eq!(a.get(), 10),
        other => panic!("Failed decode: {:?}", other),
    };
}

#[test]
fn test_pub_rel() {
    let mut data: &[u8] = &[0b01100010, 0b00000010, 0, 10];
    match decode_slice(&mut data) {
        Ok(Some(Packet::Pubrel(a))) => assert_eq!(a.get(), 10),
        other => panic!("Failed decode: {:?}", other),
    };
}

#[test]
fn test_pub_comp() {
    let mut data: &[u8] = &[0b01110000, 0b00000010, 0, 10];
    match decode_slice(&mut data) {
        Ok(Some(Packet::Pubcomp(a))) => assert_eq!(a.get(), 10),
        other => panic!("Failed decode: {:?}", other),
    };
}

#[test]
fn test_subscribe() {
    let mut data: &[u8] = &[
        0b10000010, 8, 0, 10, 0, 3, 'a' as u8, '/' as u8, 'b' as u8, 0,
    ];
    match decode_slice(&mut data) {
        Ok(Some(Packet::Subscribe(s))) => {
            assert_eq!(s.pid.get(), 10);
            let t = SubscribeTopic {
                topic_path: "a/b",
                qos: QoS::AtMostOnce,
            };
            assert_eq!(s.topics().next(), Some(t));
        }
        other => panic!("Failed decode: {:?}", other),
    }
}

#[test]
fn test_suback() {
    let mut data: &[u8] = &[0b10010000, 3, 0, 10, 0b00000010];
    match decode_slice(&mut data) {
        Ok(Some(Packet::Suback(s))) => {
            assert_eq!(s.pid.get(), 10);
            assert_eq!(
                s.return_codes().next(),
                Some(SubscribeReturnCodes::Success(QoS::ExactlyOnce))
            );
        }
        other => panic!("Failed decode: {:?}", other),
    }
}

#[test]
fn test_unsubscribe() {
    let mut data: &[u8] = &[0b10100010, 5, 0, 10, 0, 1, 'a' as u8];
    match decode_slice(&mut data) {
        Ok(Some(Packet::Unsubscribe(a))) => {
            assert_eq!(a.pid.get(), 10);
            assert_eq!(a.topics().next(), Some("a"));
        }
        other => panic!("Failed decode: {:?}", other),
    }
}

#[test]
fn test_unsub_ack() {
    let mut data: &[u8] = &[0b10110000, 2, 0, 10];
    match decode_slice(&mut data) {
        Ok(Some(Packet::Unsuback(p))) => {
            assert_eq!(p.get(), 10);
        }
        other => panic!("Failed decode: {:?}", other),
    }
}
