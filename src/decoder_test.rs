use crate::*;
use bytes::BytesMut;

fn bm(d: &[u8]) -> BytesMut {
    BytesMut::from(d)
}

#[test]
fn test_half_connect() {
    let mut data = bm(&[
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
    ]);
    assert_eq!(Ok(None), decode(&mut data));
    assert_eq!(12, data.len());
}

#[test]
fn test_connect() {
    let mut data = bm(&[
        0b00010000, 39, 0x00, 0x04, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 0x04,
        0b11001110, // +username, +password, -will retain, will qos=1, +last_will, +clean_session
        0x00, 0x0a, // 10 sec
        0x00, 0x04, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, // client_id
        0x00, 0x02, '/' as u8, 'a' as u8, // will topic = '/a'
        0x00, 0x07, 'o' as u8, 'f' as u8, 'f' as u8, 'l' as u8, 'i' as u8, 'n' as u8,
        'e' as u8, // will msg = 'offline'
        0x00, 0x04, 'r' as u8, 'u' as u8, 's' as u8, 't' as u8, // username = 'rust'
        0x00, 0x02, 'm' as u8, 'q' as u8, // password = 'mq'
    ]);
    let pkt = Connect {
        protocol: Protocol::MQTT311,
        keep_alive: 10,
        client_id: "test".into(),
        clean_session: true,
        last_will: Some(LastWill {
            topic: "/a".into(),
            message: "offline".into(),
            qos: QoS::AtLeastOnce,
            retain: false,
        }),
        username: Some("rust".into()),
        password: Some("mq".into()),
    };
    assert_eq!(Ok(Some(pkt.into())), decode(&mut data));
    assert_eq!(data.len(), 0);
}

#[test]
fn test_connack() {
    let mut data = bm(&[0b00100000, 2, 0b00000000, 0b00000001]);
    let d = decoder::decode(&mut data).unwrap();
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
    let mut data = bm(&[0b11000000, 0b00000000]);
    assert_eq!(Ok(Some(Packet::Pingreq)), decode(&mut data));
}

#[test]
fn test_ping_resp() {
    let mut data = bm(&[0b11010000, 0b00000000]);
    assert_eq!(Ok(Some(Packet::Pingresp)), decode(&mut data));
}

#[test]
fn test_disconnect() {
    let mut data = bm(&[0b11100000, 0b00000000]);
    assert_eq!(Ok(Some(Packet::Disconnect)), decode(&mut data));
}

#[test]
fn test_publish() {
    let mut data = bm(&[
        0b00110000, 10, 0x00, 0x03, 'a' as u8, '/' as u8, 'b' as u8, 'h' as u8, 'e' as u8,
        'l' as u8, 'l' as u8, 'o' as u8, //
        0b00111000, 10, 0x00, 0x03, 'a' as u8, '/' as u8, 'b' as u8, 'h' as u8, 'e' as u8,
        'l' as u8, 'l' as u8, 'o' as u8, //
        0b00111101, 12, 0x00, 0x03, 'a' as u8, '/' as u8, 'b' as u8, 0, 10, 'h' as u8, 'e' as u8,
        'l' as u8, 'l' as u8, 'o' as u8,
    ]);

    match decode(&mut data) {
        Ok(Some(Packet::Publish(p))) => {
            assert_eq!(p.dup, false);
            assert_eq!(p.retain, false);
            assert_eq!(p.qospid, QosPid::AtMostOnce);
            assert_eq!(p.topic_name, "a/b");
            assert_eq!(String::from_utf8(p.payload).unwrap(), "hello");
        }
        other => panic!("Failed decode: {:?}", other),
    }
    match decode(&mut data) {
        Ok(Some(Packet::Publish(p))) => {
            assert_eq!(p.dup, true);
            assert_eq!(p.retain, false);
            assert_eq!(p.qospid, QosPid::AtMostOnce);
            assert_eq!(p.topic_name, "a/b");
            assert_eq!(String::from_utf8(p.payload).unwrap(), "hello");
        }
        other => panic!("Failed decode: {:?}", other),
    }
    match decode(&mut data) {
        Ok(Some(Packet::Publish(p))) => {
            assert_eq!(p.dup, true);
            assert_eq!(p.retain, true);
            assert_eq!(p.qospid, QosPid::from_u8u16(2, 10));
            assert_eq!(p.topic_name, "a/b");
            assert_eq!(String::from_utf8(p.payload).unwrap(), "hello");
        }
        other => panic!("Failed decode: {:?}", other),
    }
}

#[test]
fn test_pub_ack() {
    let mut data = bm(&[0b01000000, 0b00000010, 0, 10]);
    match decode(&mut data) {
        Ok(Some(Packet::Puback(a))) => assert_eq!(a.get(), 10),
        other => panic!("Failed decode: {:?}", other),
    };
}

#[test]
fn test_pub_rec() {
    let mut data = bm(&[0b01010000, 0b00000010, 0, 10]);
    match decode(&mut data) {
        Ok(Some(Packet::Pubrec(a))) => assert_eq!(a.get(), 10),
        other => panic!("Failed decode: {:?}", other),
    };
}

#[test]
fn test_pub_rel() {
    let mut data = bm(&[0b01100010, 0b00000010, 0, 10]);
    match decode(&mut data) {
        Ok(Some(Packet::Pubrel(a))) => assert_eq!(a.get(), 10),
        other => panic!("Failed decode: {:?}", other),
    };
}

#[test]
fn test_pub_comp() {
    let mut data = bm(&[0b01110000, 0b00000010, 0, 10]);
    match decode(&mut data) {
        Ok(Some(Packet::Pubcomp(a))) => assert_eq!(a.get(), 10),
        other => panic!("Failed decode: {:?}", other),
    };
}

#[test]
fn test_subscribe() {
    let mut data = bm(&[
        0b10000010, 8, 0, 10, 0, 3, 'a' as u8, '/' as u8, 'b' as u8, 0,
    ]);
    match decode(&mut data) {
        Ok(Some(Packet::Subscribe(s))) => {
            assert_eq!(s.pid.get(), 10);
            let t = SubscribeTopic {
                topic_path: "a/b".to_string(),
                qos: QoS::AtMostOnce,
            };
            assert_eq!(s.topics[0], t);
        }
        other => panic!("Failed decode: {:?}", other),
    }
}

#[test]
fn test_suback() {
    let mut data = bm(&[0b10010000, 3, 0, 10, 0b00000010]);
    match decode(&mut data) {
        Ok(Some(Packet::Suback(s))) => {
            assert_eq!(s.pid.get(), 10);
            assert_eq!(
                s.return_codes[0],
                SubscribeReturnCodes::Success(QoS::ExactlyOnce)
            );
        }
        other => panic!("Failed decode: {:?}", other),
    }
}

#[test]
fn test_unsubscribe() {
    let mut data = bm(&[0b10100010, 5, 0, 10, 0, 1, 'a' as u8]);
    match decode(&mut data) {
        Ok(Some(Packet::Unsubscribe(a))) => {
            assert_eq!(a.pid.get(), 10);
            assert_eq!(a.topics[0], 'a'.to_string());
        }
        other => panic!("Failed decode: {:?}", other),
    }
}

#[test]
fn test_unsub_ack() {
    let mut data = bm(&[0b10110000, 2, 0, 10]);
    match decode(&mut data) {
        Ok(Some(Packet::Unsuback(p))) => {
            assert_eq!(p.get(), 10);
        }
        other => panic!("Failed decode: {:?}", other),
    }
}
