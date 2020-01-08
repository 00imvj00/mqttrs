use crate::{Error, Packet};
use bytes::{BufMut, BytesMut};

/// Encode a [Packet] enum into a [BytesMut] buffer.
///
/// ```
/// # use mqttrs::*;
/// # use bytes::*;
/// // Instantiate a `Packet` to encode.
/// let packet = Publish {
///    dup: false,
///    qospid: QosPid::AtMostOnce,
///    retain: false,
///    topic_name: "test".into(),
///    payload: "hello".into(),
/// }.into();
///
/// // Allocate buffer (should be appropriately-sized or able to grow as needed).
/// let mut buf = BytesMut::with_capacity(1024);
///
/// // Write bytes corresponding to `&Packet` into the `BytesMut`.
/// encode(&packet, &mut buf).expect("failed encoding");
/// assert_eq!(&*buf, &[0b00110000, 11,
///                     0, 4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8,
///                    'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8]);
/// ```
///
/// [Packet]: ../enum.Packet.html
/// [BytesMut]: https://docs.rs/bytes/0.5.3/bytes/struct.BytesMut.html
pub fn encode(packet: &Packet, buf: &mut BytesMut) -> Result<(), Error> {
    match packet {
        Packet::Connect(connect) => connect.to_buffer(buf),
        Packet::Connack(connack) => connack.to_buffer(buf),
        Packet::Publish(publish) => publish.to_buffer(buf),
        Packet::Puback(pid) => {
            check_remaining(buf, 4)?;
            let header: u8 = 0b01000000;
            let length: u8 = 2;
            buf.put_u8(header);
            buf.put_u8(length);
            pid.to_buffer(buf)
        }
        Packet::Pubrec(pid) => {
            check_remaining(buf, 4)?;
            let header: u8 = 0b01010000;
            let length: u8 = 2;
            buf.put_u8(header);
            buf.put_u8(length);
            pid.to_buffer(buf)
        }
        Packet::Pubrel(pid) => {
            check_remaining(buf, 4)?;
            let header: u8 = 0b01100010;
            let length: u8 = 2;
            buf.put_u8(header);
            buf.put_u8(length);
            pid.to_buffer(buf)
        }
        Packet::Pubcomp(pid) => {
            check_remaining(buf, 4)?;
            let header: u8 = 0b01110000;
            let length: u8 = 2;
            buf.put_u8(header);
            buf.put_u8(length);
            pid.to_buffer(buf)
        }
        Packet::Subscribe(subscribe) => subscribe.to_buffer(buf),
        Packet::Suback(suback) => suback.to_buffer(buf),
        Packet::Unsubscribe(unsub) => unsub.to_buffer(buf),
        Packet::Unsuback(pid) => {
            check_remaining(buf, 4)?;
            let header: u8 = 0b10110000;
            let length: u8 = 2;
            buf.put_u8(header);
            buf.put_u8(length);
            pid.to_buffer(buf)
        }
        Packet::Pingreq => {
            check_remaining(buf, 2)?;
            let header: u8 = 0b11000000;
            let length: u8 = 0;
            buf.put_u8(header);
            buf.put_u8(length);
            Ok(())
        }
        Packet::Pingresp => {
            check_remaining(buf, 2)?;
            let header: u8 = 0b11010000;
            let length: u8 = 0;
            buf.put_u8(header);
            buf.put_u8(length);
            Ok(())
        }
        Packet::Disconnect => {
            check_remaining(buf, 2)?;
            let header: u8 = 0b11100000;
            let length: u8 = 0;
            buf.put_u8(header);
            buf.put_u8(length);
            Ok(())
        }
    }
}

/// Check wether buffer has `len` bytes of write capacity left. Use this to return a clean
/// Result::Err instead of panicking.
pub(crate) fn check_remaining(buf: &BytesMut, len: usize) -> Result<(), Error> {
    if buf.remaining_mut() < len {
        Err(Error::WriteZero)
    } else {
        Ok(())
    }
}

/// http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718023
pub(crate) fn write_length(len: usize, buf: &mut BytesMut) -> Result<(), Error> {
    match len {
        0..=127 => check_remaining(buf, len + 1)?,
        128..=16383 => check_remaining(buf, len + 2)?,
        16384..=2097151 => check_remaining(buf, len + 3)?,
        2097152..=268435455 => check_remaining(buf, len + 4)?,
        _ => return Err(Error::InvalidLength),
    }
    let mut done = false;
    let mut x = len;
    while !done {
        let mut byte = (x % 128) as u8;
        x = x / 128;
        if x > 0 {
            byte = byte | 128;
        }
        buf.put_u8(byte);
        done = x <= 0;
    }
    Ok(())
}

pub(crate) fn write_bytes(bytes: &[u8], buf: &mut BytesMut) -> Result<(), Error> {
    buf.put_u16(bytes.len() as u16);
    buf.put_slice(bytes);
    Ok(())
}

pub(crate) fn write_string(string: &str, buf: &mut BytesMut) -> Result<(), Error> {
    write_bytes(string.as_bytes(), buf)
}
