use crate::{decoder, Connack, ConnectReturnCode, Packet, QoS};
use bytes::BytesMut;

#[test]
fn test_half_connect() {
    let mut data = BytesMut::from(vec![
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
    let length = data.len();
    let d = decoder::decode(&mut data).unwrap();
    assert_eq!(d, None);
    assert_eq!(length, 12);
}

#[test]
fn test_connect() {
    let mut data = BytesMut::from(vec![
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
    let d = decoder::decode(&mut data).unwrap();
    assert_ne!(d, None);
    assert_eq!(data.len(), 0);
}

#[test]
fn test_connack() {
    //let mut data = BytesMut::from(vec![0b00100000, 2, 0b00000001, 0b00000000]);
    let mut data = BytesMut::from(vec![0b00100000, 2, 0b00000000, 0b00000001]);
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
    let mut data = BytesMut::from(vec![0b11000000, 0b00000000]);
    let d = decoder::decode(&mut data).unwrap();
    assert_eq!(d, Some(Packet::PingReq));
}

#[test]
fn test_ping_resp() {
    let mut data = BytesMut::from(vec![0b11010000, 0b00000000]);
    let d = decoder::decode(&mut data).unwrap();
    assert_eq!(d, Some(Packet::PingResp));
}

#[test]
fn test_disconnect() {
    let mut data = BytesMut::from(vec![0b11100000, 0b00000000]);
    let d = decoder::decode(&mut data).unwrap();
    assert_eq!(d, Some(Packet::Disconnect));
}

#[test]
fn test_publish() {
    let mut data = BytesMut::from(vec![
        0b00110000, 12, 0x00, 0x03, 'a' as u8, '/' as u8, 'b' as u8, 0x00, 0x10, 'h' as u8,
        'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0b00111000, 12, 0x00, 0x03, 'a' as u8,
        '/' as u8, 'b' as u8, 0x00, 0x10, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8,
        0b00111101, 12, 0x00, 0x03, 'a' as u8, '/' as u8, 'b' as u8, 0x00, 0x10, 'h' as u8,
        'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8,
    ]);
    let d1 = decoder::decode(&mut data).unwrap();
    let d2 = decoder::decode(&mut data).unwrap();
    let d3 = decoder::decode(&mut data).unwrap();
    match d1 {
        Some(Packet::Publish(p)) => {
            assert_eq!(p.dup, false);
            assert_eq!(p.retain, false);
            assert_eq!(p.qos, QoS::AtMostOnce);
            assert_eq!(p.topic_name, "a/b");
            assert_eq!(String::from_utf8(p.payload).unwrap(), "hello");
        }
        _ => panic!("Should not be None"),
    }
    match d2 {
        Some(Packet::Publish(p)) => {
            assert_eq!(p.dup, true);
            assert_eq!(p.retain, false);
            assert_eq!(p.qos, QoS::AtMostOnce);
            assert_eq!(p.topic_name, "a/b");
            assert_eq!(String::from_utf8(p.payload).unwrap(), "hello");
        }
        _ => panic!("Should not be None"),
    }
    match d3 {
        Some(Packet::Publish(p)) => {
            assert_eq!(p.dup, true);
            assert_eq!(p.retain, true);
            assert_eq!(p.qos, QoS::ExactlyOnce);
            assert_eq!(p.topic_name, "a/b");
            assert_eq!(String::from_utf8(p.payload).unwrap(), "hello");
        }
        _ => panic!("Should not be None"),
    }
}
