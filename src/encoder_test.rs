use crate::*;
use bytes::BytesMut;

macro_rules! assert_decode {
    ($res:pat, $pkt:expr) => {
        let mut buf = BytesMut::with_capacity(1024);
        encode($pkt, &mut buf).unwrap();
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
    };
    assert_decode!(Packet::Connect(_), &packet.into());
}

#[test]
fn test_connack() {
    let packet = Connack {
        session_present: true,
        code: ConnectReturnCode::Accepted,
    };
    assert_decode!(Packet::Connack(_), &packet.into());
}

#[test]
fn test_publish() {
    let packet = Publish {
        dup: false,
        qospid: QosPid::from_u8u16(2, 10),
        retain: true,
        topic_name: "asdf".to_string(),
        payload: vec!['h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8],
    };
    assert_decode!(Packet::Publish(_), &packet.into());
}

#[test]
fn test_puback() {
    let packet = Packet::Puback(Pid::try_from(19).unwrap());
    assert_decode!(Packet::Puback(_), &packet);
}

#[test]
fn test_pubrec() {
    let packet = Packet::Pubrec(Pid::try_from(19).unwrap());
    assert_decode!(Packet::Pubrec(_), &packet);
}

#[test]
fn test_pubrel() {
    let packet = Packet::Pubrel(Pid::try_from(19).unwrap());
    assert_decode!(Packet::Pubrel(_), &packet);
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
    };
    assert_decode!(Packet::Subscribe(_), &Packet::Subscribe(packet));
}

#[test]
fn test_suback() {
    let return_code = SubscribeReturnCodes::Success(QoS::ExactlyOnce);
    let packet = Suback {
        pid: Pid::try_from(12321).unwrap(),
        return_codes: vec![return_code],
    };
    assert_decode!(Packet::Suback(_), &Packet::Suback(packet));
}

#[test]
fn test_unsubscribe() {
    let packet = Unsubscribe {
        pid: Pid::try_from(12321).unwrap(),
        topics: vec!["a/b".to_string()],
    };
    assert_decode!(Packet::Unsubscribe(_), &Packet::Unsubscribe(packet));
}

#[test]
fn test_unsuback() {
    let packet = Packet::Unsuback(Pid::try_from(19).unwrap());
    assert_decode!(Packet::Unsuback(_), &packet);
}

#[test]
fn test_ping_req() {
    assert_decode!(Packet::Pingreq, &Packet::Pingreq);
}

#[test]
fn test_ping_resp() {
    assert_decode!(Packet::Pingresp, &Packet::Pingresp);
}

#[test]
fn test_disconnect() {
    assert_decode!(Packet::Disconnect, &Packet::Disconnect);
}
