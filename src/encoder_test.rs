use crate::*;
use bytes::BytesMut;
use core::convert::TryFrom;
use alloc::string::ToString;
use alloc::vec;

macro_rules! assert_decode {
    ($res:pat, $pkt:expr) => {
        let mut buf = BytesMut::with_capacity(1024);
        let written = encode($pkt, &mut buf).unwrap();
        assert_eq!(written, buf.len());
        match decode(&mut buf) {
            Ok(Some($res)) => (),
            err => assert!(
                false,
                "Expected: Ok(Some({}))  got: {:?}",
                stringify!($res),
                err
            ),
        }
    };
}
macro_rules! assert_decode_slice {
    ($res:pat, $pkt:expr) => {
        let mut slice = [0u8; 1024];
        let written = encode($pkt, &mut slice[..]).unwrap();
        match decode(&mut &slice[..written]) {
            Ok(Some($res)) => (),
            err => assert!(
                false,
                "Expected: Ok(Some({}))  got: {:?}",
                stringify!($res),
                err
            ),
        }
    };
}

#[test]
fn test_connect() {
    let packet = Connect {
        protocol: Protocol::new("MQTT", 4).unwrap(),
        keep_alive: 120,
        client_id: "imvj".to_string(),
        clean_session: true,
        last_will: None,
        username: None,
        password: None,
    }.into();
    assert_decode!(Packet::Connect(_), &packet);
    assert_decode_slice!(Packet::Connect(_), &packet);
}

#[test]
fn test_write_zero() {
    let packet = Connect {
        protocol: Protocol::new("MQTT", 4).unwrap(),
        keep_alive: 120,
        client_id: "imvj".to_string(),
        clean_session: true,
        last_will: None,
        username: None,
        password: None,
    }.into();

    let mut slice = [0u8; 8];
    match encode(&packet, &mut slice[..]) {
        Ok(_) => panic!("Expected Error::WriteZero, as input slice is too small"),
        Err(e) => {
            assert_eq!(e, Error::WriteZero)
        }
    }

    let mut buf = BytesMut::with_capacity(8);
    let written = encode(&packet, &mut buf).unwrap();
    assert_eq!(written, buf.len());
    assert_eq!(buf.len(), 18);
}

#[test]
fn test_connack() {
    let packet = Connack {
        session_present: true,
        code: ConnectReturnCode::Accepted,
    }.into();
    assert_decode!(Packet::Connack(_), &packet);
    assert_decode_slice!(Packet::Connack(_), &packet);
}

#[test]
fn test_publish() {
    let packet = Publish {
        dup: false,
        qospid: QosPid::from_u8u16(2, 10),
        retain: true,
        topic_name: "asdf".to_string(),
        payload: vec!['h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8],
    }.into();
    assert_decode!(Packet::Publish(_), &packet);
    assert_decode_slice!(Packet::Publish(_), &packet);
}

#[test]
fn test_puback() {
    let packet = Packet::Puback(Pid::try_from(19).unwrap());
    assert_decode!(Packet::Puback(_), &packet);
    assert_decode_slice!(Packet::Puback(_), &packet);
}

#[test]
fn test_pubrec() {
    let packet = Packet::Pubrec(Pid::try_from(19).unwrap());
    assert_decode!(Packet::Pubrec(_), &packet);
    assert_decode_slice!(Packet::Pubrec(_), &packet);
}

#[test]
fn test_pubrel() {
    let packet = Packet::Pubrel(Pid::try_from(19).unwrap());
    assert_decode!(Packet::Pubrel(_), &packet);
    assert_decode_slice!(Packet::Pubrel(_), &packet);
}

#[test]
fn test_pubcomp() {
    let packet = Packet::Pubcomp(Pid::try_from(19).unwrap());
    assert_decode!(Packet::Pubcomp(_), &packet);
}

#[test]
fn test_subscribe() {
    let stopic = SubscribeTopic {
        topic_path: "a/b".to_string(),
        qos: QoS::ExactlyOnce,
    };
    let packet = Subscribe {
        pid: Pid::try_from(345).unwrap(),
        topics: vec![stopic],
    }.into();
    assert_decode!(Packet::Subscribe(_), &packet);
    assert_decode_slice!(Packet::Subscribe(_), &packet);
}

#[test]
fn test_suback() {
    let return_code = SubscribeReturnCodes::Success(QoS::ExactlyOnce);
    let packet = Suback {
        pid: Pid::try_from(12321).unwrap(),
        return_codes: vec![return_code],
    }.into();
    assert_decode!(Packet::Suback(_), &packet);
    assert_decode_slice!(Packet::Suback(_), &packet);
}

#[test]
fn test_unsubscribe() {
    let packet = Unsubscribe {
        pid: Pid::try_from(12321).unwrap(),
        topics: vec!["a/b".to_string()],
    }.into();
    assert_decode!(Packet::Unsubscribe(_), &packet);
    assert_decode_slice!(Packet::Unsubscribe(_), &packet);
}

#[test]
fn test_unsuback() {
    let packet = Packet::Unsuback(Pid::try_from(19).unwrap());
    assert_decode!(Packet::Unsuback(_), &packet);
    assert_decode_slice!(Packet::Unsuback(_), &packet);
}

#[test]
fn test_ping_req() {
    assert_decode!(Packet::Pingreq, &Packet::Pingreq);
    assert_decode_slice!(Packet::Pingreq, &Packet::Pingreq);
}

#[test]
fn test_ping_resp() {
    assert_decode!(Packet::Pingresp, &Packet::Pingresp);
    assert_decode_slice!(Packet::Pingresp, &Packet::Pingresp);
}

#[test]
fn test_disconnect() {
    assert_decode!(Packet::Disconnect, &Packet::Disconnect);
    assert_decode_slice!(Packet::Disconnect, &Packet::Disconnect);
}
