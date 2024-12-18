use crate::*;
use core::convert::TryFrom;
use subscribe::{LimitedString, LimitedVec};
use core::str::FromStr;

#[cfg(feature = "std")]
use bytes::BytesMut;

// macro_rules! assert_decode {
//     ($res:pat, $pkt:expr) => {
//         let mut buf = BytesMut::with_capacity(1024);
//         let written = encode($pkt, &mut buf).unwrap();
//         assert_eq!(written, buf.len());
//         match decode_slice(&mut buf) {
//             Ok(Some($res)) => (),
//             err => assert!(
//                 false,
//                 "Expected: Ok(Some({}))  got: {:?}",
//                 stringify!($res),
//                 err
//             ),
//         }
//     };
// }
macro_rules! assert_decode_slice {
    ($res:pat, $pkt:expr, $written_exp:expr) => {
        let mut slice = [0u8; 512];
        let written = encode_slice($pkt, &mut slice).unwrap();
        assert_eq!(written, $written_exp);
        match decode_slice(&slice[..written]) {
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
        client_id: "imvj",
        clean_session: true,
        last_will: None,
        username: None,
        password: None,
    }
    .into();
    // assert_decode!(Packet::Connect(_), &packet);
    assert_decode_slice!(Packet::Connect(_), &packet, 18);
}

#[test]
fn test_write_zero() {
    let packet = Connect {
        protocol: Protocol::new("MQTT", 4).unwrap(),
        keep_alive: 120,
        client_id: "imvj",
        clean_session: true,
        last_will: None,
        username: None,
        password: None,
    }
    .into();

    let mut slice = [0u8; 8];
    match encode_slice(&packet, &mut slice) {
        Ok(_) => panic!("Expected Error::WriteZero, as input slice is too small"),
        Err(e) => assert_eq!(e, Error::WriteZero),
    }

    let mut buf = [0u8; 80];
    let written = encode_slice(&packet, &mut buf).unwrap();
    assert_eq!(written, 18);
}

#[test]
fn test_connack() {
    let packet = Connack {
        session_present: true,
        code: ConnectReturnCode::Accepted,
    }
    .into();
    // assert_decode!(Packet::Connack(_), &packet);
    assert_decode_slice!(Packet::Connack(_), &packet, 4);
}

#[test]
fn test_publish() {
    let packet = Publish {
        dup: false,
        qospid: QosPid::from_u8u16(2, 10),
        retain: true,
        topic_name: "asdf",
        payload: &['h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8],
    }
    .into();
    // assert_decode!(Packet::Publish(_), &packet);
    assert_decode_slice!(Packet::Publish(_), &packet, 15);
}

#[test]
fn test_puback() {
    let packet = Packet::Puback(Pid::try_from(19).unwrap());
    // assert_decode!(Packet::Puback(_), &packet);
    assert_decode_slice!(Packet::Puback(_), &packet, 4);
}

#[test]
fn test_pubrec() {
    let packet = Packet::Pubrec(Pid::try_from(19).unwrap());
    // assert_decode!(Packet::Pubrec(_), &packet);
    assert_decode_slice!(Packet::Pubrec(_), &packet, 4);
}

#[test]
fn test_pubrel() {
    let packet = Packet::Pubrel(Pid::try_from(19).unwrap());
    // assert_decode!(Packet::Pubrel(_), &packet);
    assert_decode_slice!(Packet::Pubrel(_), &packet, 4);
}

#[test]
fn test_pubcomp() {
    let packet = Packet::Pubcomp(Pid::try_from(19).unwrap());
    // assert_decode!(Packet::Pubcomp(_), &packet);
    assert_decode_slice!(Packet::Pubcomp(_), &packet, 4);
}
#[cfg(feature = "std")]
#[test]
fn test_subscribe() {
    let stopic = SubscribeTopic {
        topic_path: LimitedString::from("a/b"),
        qos: QoS::ExactlyOnce,
    };
    let topics: LimitedVec<SubscribeTopic> = [stopic].iter().cloned().collect();
    let packet = Subscribe::new(Pid::try_from(345).unwrap(), topics).into();
    // assert_decode!(Packet::Subscribe(_), &packet);
    assert_decode_slice!(Packet::Subscribe(_), &packet, 10);
}

#[cfg(not(feature = "std"))]
#[test]
fn test_subscribe() {
    let stopic = SubscribeTopic {
        topic_path: LimitedString::from_str("a/b").unwrap(),
        qos: QoS::ExactlyOnce,
    };
    let topics: LimitedVec<SubscribeTopic> = [stopic].iter().cloned().collect();
    let packet = Subscribe::new(Pid::try_from(345).unwrap(), topics).into();
    // assert_decode!(Packet::Subscribe(_), &packet);
    assert_decode_slice!(Packet::Subscribe(_), &packet, 10);
}

#[test]
fn test_suback() {
    let return_codes = [SubscribeReturnCodes::Success(QoS::ExactlyOnce)]
        .iter()
        .cloned()
        .collect();
    let packet = Suback::new(Pid::try_from(12321).unwrap(), return_codes).into();
    // assert_decode!(Packet::Suback(_), &packet);
    assert_decode_slice!(Packet::Suback(_), &packet, 5);
}

#[test]
fn test_unsubscribe() {
    #[cfg(feature = "std")]
    let topics: LimitedVec<LimitedString> = [LimitedString::from("a/b")].iter().cloned().collect();
    #[cfg(not(feature = "std"))]
    let topics: LimitedVec<LimitedString> = [LimitedString::from_str("a/b").unwrap()].iter().cloned().collect();

    let packet = Unsubscribe::new(Pid::try_from(12321).unwrap(), topics).into();
    // assert_decode!(Packet::Unsubscribe(_), &packet);
    assert_decode_slice!(Packet::Unsubscribe(_), &packet, 9);
}

#[test]
fn test_unsuback() {
    let packet = Packet::Unsuback(Pid::try_from(19).unwrap());
    // assert_decode!(Packet::Unsuback(_), &packet);
    assert_decode_slice!(Packet::Unsuback(_), &packet, 4);
}

#[test]
fn test_ping_req() {
    // assert_decode!(Packet::Pingreq, &Packet::Pingreq);
    assert_decode_slice!(Packet::Pingreq, &Packet::Pingreq, 2);
}

#[test]
fn test_ping_resp() {
    // assert_decode!(Packet::Pingresp, &Packet::Pingresp);
    assert_decode_slice!(Packet::Pingresp, &Packet::Pingresp, 2);
}

#[test]
fn test_disconnect() {
    // assert_decode!(Packet::Disconnect, &Packet::Disconnect);
    assert_decode_slice!(Packet::Disconnect, &Packet::Disconnect, 2);
}
