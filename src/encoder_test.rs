use crate::*;
use bytes::BytesMut;

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
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&packet.into(), &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Connect(_))) => assert!(true),
        err => assert!(false, err),
    }
}

#[test]
fn test_connack() {
    let packet = Connack {
        session_present: true,
        code: ConnectReturnCode::Accepted,
    };
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&packet.into(), &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Connack(_))) => assert!(true),
        err => assert!(false, err),
    }
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
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&packet.into(), &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Publish(_))) => assert!(true),
        err => assert!(false, err),
    }
}

#[test]
fn test_puback() {
    let packet = Packet::Puback(Pid::new(19).unwrap());
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&packet, &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Puback(_))) => assert!(true),
        err => assert!(false, err),
    }
}

#[test]
fn test_pubrec() {
    let packet = Packet::Pubrec(Pid::new(19).unwrap());
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&packet, &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Pubrec(_))) => assert!(true),
        err => assert!(false, err),
    }
}

#[test]
fn test_pubrel() {
    let packet = Packet::Pubrel(Pid::new(19).unwrap());
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&packet, &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Pubrel(_))) => assert!(true),
        err => assert!(false, err),
    }
}

#[test]
fn test_pubcomp() {
    let packet = Packet::Pubcomp(Pid::new(19).unwrap());
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&packet, &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Pubcomp(_))) => assert!(true),
        err => assert!(false, err),
    }
}

#[test]
fn test_subscribe() {
    let stopic = SubscribeTopic {
        topic_path: "a/b".to_string(),
        qos: QoS::ExactlyOnce,
    };
    let packet = Subscribe {
        pid: Pid::new(345).unwrap(),
        topics: vec![stopic],
    };
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&Packet::Subscribe(packet), &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Subscribe(_))) => assert!(true),
        err => assert!(false, err),
    }
}

#[test]
fn test_suback() {
    let return_code = SubscribeReturnCodes::Success(QoS::ExactlyOnce);
    let packet = Suback {
        pid: Pid::new(12321).unwrap(),
        return_codes: vec![return_code],
    };
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&Packet::Suback(packet), &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Suback(_))) => assert!(true),
        err => assert!(false, err),
    }
}

#[test]
fn test_unsubscribe() {
    let packet = Unsubscribe {
        pid: Pid::new(12321).unwrap(),
        topics: vec!["a/b".to_string()],
    };
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&Packet::Unsubscribe(packet), &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Unsubscribe(_))) => assert!(true),
        err => assert!(false, err),
    }
}

#[test]
fn test_unsuback() {
    let packet = Packet::Unsuback(Pid::new(19).unwrap());
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&packet, &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Unsuback(_))) => assert!(true),
        err => assert!(false, err),
    }
}

#[test]
fn test_ping_req() {
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&Packet::Pingreq, &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Pingreq)) => assert!(true),
        err => assert!(false, err),
    }
}

#[test]
fn test_ping_resp() {
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&Packet::Pingresp, &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Pingresp)) => assert!(true),
        err => assert!(false, err),
    }
}

#[test]
fn test_disconnect() {
    let mut buffer = BytesMut::with_capacity(1024);
    encode(&Packet::Disconnect, &mut buffer).unwrap();
    match decode(&mut buffer) {
        Ok(Some(Packet::Disconnect)) => assert!(true),
        err => assert!(false, err),
    }
}
