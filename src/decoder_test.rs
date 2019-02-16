use crate::decoder;
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
    let d = decoder::decode(&mut data);
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
    let d = decoder::decode(&mut data);
    assert_ne!(d, None);
    assert_eq!(data.len(), 0);
}
