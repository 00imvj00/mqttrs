use crate::{Error, Packet};

/// Encode a [Packet] enum into a [BufMut] buffer.
///
/// ```
/// # use mqttrs::*;
/// # use bytes::*;
/// // Instantiate a `Packet` to encode.
/// let packet = Publish {
///    dup: false,
///    qospid: QosPid::AtMostOnce,
///    retain: false,
///    topic_name: "test",
///    payload: b"hello",
/// }.into();
///
/// // Allocate buffer (should be appropriately-sized or able to grow as needed).
/// let mut buf = [0u8; 1024];
///
/// // Write bytes corresponding to `&Packet` into the `BytesMut`.
/// let len = encode_slice(&packet, &mut buf).expect("failed encoding");
/// assert_eq!(&buf[..len], &[0b00110000, 11,
///                     0, 4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8,
///                    'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8]);
/// ```
///
/// [Packet]: ../enum.Packet.html
/// [BufMut]: https://docs.rs/bytes/0.5.3/bytes/trait.BufMut.html
// #[cfg(feature = "std")]
// pub fn encode_slice(packet: &Packet, buf: impl BufMut) -> Result<usize, Error> {
//     let mut offset = 0;
//     encode_slice(packet, buf.bytes_mut(), &mut offset)
// }

pub fn encode_slice(packet: &Packet, buf: &mut [u8]) -> Result<usize, Error> {
    let mut offset = 0;

    match packet {
        Packet::Connect(connect) => connect.to_buffer(buf, &mut offset),
        Packet::Connack(connack) => connack.to_buffer(buf, &mut offset),
        Packet::Publish(publish) => publish.to_buffer(buf, &mut offset),
        Packet::Puback(pid) => {
            check_remaining(buf, &mut offset, 4)?;
            let header: u8 = 0b01000000;
            let length: u8 = 2;
            write_u8(buf, &mut offset, header)?;
            write_u8(buf, &mut offset, length)?;
            pid.to_buffer(buf, &mut offset)?;
            Ok(4)
        }
        Packet::Pubrec(pid) => {
            check_remaining(buf, &mut offset, 4)?;
            let header: u8 = 0b01010000;
            let length: u8 = 2;
            write_u8(buf, &mut offset, header)?;
            write_u8(buf, &mut offset, length)?;
            pid.to_buffer(buf, &mut offset)?;
            Ok(4)
        }
        Packet::Pubrel(pid) => {
            check_remaining(buf, &mut offset, 4)?;
            let header: u8 = 0b01100010;
            let length: u8 = 2;
            write_u8(buf, &mut offset, header)?;
            write_u8(buf, &mut offset, length)?;
            pid.to_buffer(buf, &mut offset)?;
            Ok(4)
        }
        Packet::Pubcomp(pid) => {
            check_remaining(buf, &mut offset, 4)?;
            let header: u8 = 0b01110000;
            let length: u8 = 2;
            write_u8(buf, &mut offset, header)?;
            write_u8(buf, &mut offset, length)?;
            pid.to_buffer(buf, &mut offset)?;
            Ok(4)
        }
        Packet::Subscribe(subscribe) => subscribe.to_buffer(buf, &mut offset),
        Packet::Suback(suback) => suback.to_buffer(buf, &mut offset),
        Packet::Unsubscribe(unsub) => unsub.to_buffer(buf, &mut offset),
        Packet::Unsuback(pid) => {
            check_remaining(buf, &mut offset, 4)?;
            let header: u8 = 0b10110000;
            let length: u8 = 2;
            write_u8(buf, &mut offset, header)?;
            write_u8(buf, &mut offset, length)?;
            pid.to_buffer(buf, &mut offset)?;
            Ok(4)
        }
        Packet::Pingreq => {
            check_remaining(buf, &mut offset, 2)?;
            let header: u8 = 0b11000000;
            let length: u8 = 0;
            write_u8(buf, &mut offset, header)?;
            write_u8(buf, &mut offset, length)?;
            Ok(2)
        }
        Packet::Pingresp => {
            check_remaining(buf, &mut offset, 2)?;
            let header: u8 = 0b11010000;
            let length: u8 = 0;
            write_u8(buf, &mut offset, header)?;
            write_u8(buf, &mut offset, length)?;
            Ok(2)
        }
        Packet::Disconnect => {
            check_remaining(buf, &mut offset, 2)?;
            let header: u8 = 0b11100000;
            let length: u8 = 0;
            write_u8(buf, &mut offset, header)?;
            write_u8(buf, &mut offset, length)?;
            Ok(2)
        }
    }
}

/// Check wether buffer has `len` bytes of write capacity left. Use this to return a clean
/// Result::Err instead of panicking.
pub(crate) fn check_remaining(buf: &mut [u8], offset: &mut usize, len: usize) -> Result<(), Error> {
    if buf[*offset..].len() < len {
        Err(Error::WriteZero)
    } else {
        Ok(())
    }
}

/// http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718023
pub(crate) fn write_length(buf: &mut [u8], offset: &mut usize, len: usize) -> Result<usize, Error> {
    let write_len = match len {
        0..=127 => {
            check_remaining(buf, offset, len + 1)?;
            len + 1
        }
        128..=16383 => {
            check_remaining(buf, offset, len + 2)?;
            len + 2
        }
        16384..=2097151 => {
            check_remaining(buf, offset, len + 3)?;
            len + 3
        }
        2097152..=268435455 => {
            check_remaining(buf, offset, len + 4)?;
            len + 4
        }
        _ => return Err(Error::InvalidLength),
    };
    let mut done = false;
    let mut x = len;
    while !done {
        let mut byte = (x % 128) as u8;
        x = x / 128;
        if x > 0 {
            byte = byte | 128;
        }
        write_u8(buf, offset, byte)?;
        done = x <= 0;
    }
    Ok(write_len)
}


pub(crate) fn write_u8(buf: &mut [u8], offset: &mut usize, val: u8) -> Result<(), Error> {
    buf[*offset] = val;
    *offset += 1;
    Ok(())
}

pub(crate) fn write_u16(buf: &mut [u8], offset: &mut usize, val: u16) -> Result<(), Error> {
    write_u8(buf, offset, (val >> 8) as u8)?;
    write_u8(buf, offset, (val & 0xFF) as u8)
}

pub(crate) fn write_bytes(buf: &mut [u8], offset: &mut usize, bytes: &[u8]) -> Result<(), Error> {
    write_u16(buf, offset, bytes.len() as u16)?;

    for &byte in bytes {
        write_u8(buf, offset, byte)?;
    }
    Ok(())
}

pub(crate) fn write_string(buf: &mut [u8], offset: &mut usize, string: &str) -> Result<(), Error> {
    write_bytes(buf, offset, string.as_bytes(), )
}
