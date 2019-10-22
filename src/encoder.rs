use crate::{Error, Packet};
use bytes::{BufMut, BytesMut};

/// Encode a [Packet] enum into a buffer.
///
/// [Packet]: ../enum.Packet.html
pub fn encode(packet: &Packet, buffer: &mut BytesMut) -> Result<(), Error> {
    match packet {
        Packet::Connect(connect) => connect.to_buffer(buffer),
        Packet::Connack(connack) => connack.to_buffer(buffer),
        Packet::Publish(publish) => publish.to_buffer(buffer),
        Packet::Puback(pid) => {
            check_remaining(buffer, 4)?;
            let header_u8 = 0b01000000 as u8;
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            pid.to_buffer(buffer)
        }
        Packet::Pubrec(pid) => {
            check_remaining(buffer, 4)?;
            let header_u8 = 0b01010000 as u8;
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            pid.to_buffer(buffer)
        }
        Packet::Pubrel(pid) => {
            check_remaining(buffer, 4)?;
            let header_u8 = 0b01100000 as u8;
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            pid.to_buffer(buffer)
        }
        Packet::Pubcomp(pid) => {
            check_remaining(buffer, 4)?;
            let header_u8 = 0b01110000 as u8;
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            pid.to_buffer(buffer)
        }
        Packet::Subscribe(subscribe) => subscribe.to_buffer(buffer),
        Packet::Suback(suback) => suback.to_buffer(buffer),
        Packet::Unsubscribe(unsub) => unsub.to_buffer(buffer),
        Packet::Unsuback(pid) => {
            check_remaining(buffer, 4)?;
            let header_u8 = 0b10110000 as u8;
            let length = 0b00000010 as u8;
            buffer.put(header_u8);
            buffer.put(length);
            pid.to_buffer(buffer)
        }
        Packet::Pingreq => {
            check_remaining(buffer, 2)?;
            buffer.put(0b11000000 as u8);
            buffer.put(0b00000000 as u8);
            Ok(())
        }
        Packet::Pingresp => {
            check_remaining(buffer, 2)?;
            buffer.put(0b11010000 as u8);
            buffer.put(0b00000000 as u8);
            Ok(())
        }
        Packet::Disconnect => {
            check_remaining(buffer, 2)?;
            buffer.put(0b11100000 as u8);
            buffer.put(0b00000000 as u8);
            Ok(())
        }
    }
}

/// Check wether buffer has `len` bytes of write capacity left. Use this to return a clean
/// Result::Err instead of panicking.
pub(crate) fn check_remaining(buffer: &BytesMut, len: usize) -> Result<(), Error> {
    if buffer.remaining_mut() < len {
        Err(Error::WriteZero)
    } else {
        Ok(())
    }
}

/// http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718023
pub(crate) fn write_length(len: usize, buffer: &mut BytesMut) -> Result<(), Error> {
    match len {
        0..=127 => check_remaining(buffer, len + 1)?,
        128..=16383 => check_remaining(buffer, len + 2)?,
        16384..=2097151 => check_remaining(buffer, len + 3)?,
        2097152..=268435455 => check_remaining(buffer, len + 4)?,
        _ => return Err(Error::InvalidLength(len)),
    }
    let mut done = false;
    let mut x = len;
    while !done {
        let mut byte = (x % 128) as u8;
        x = x / 128;
        if x > 0 {
            byte = byte | 128;
        }
        buffer.put(byte as u8);
        done = x <= 0;
    }
    Ok(())
}

pub(crate) fn write_bytes(bytes: &[u8], buffer: &mut BytesMut) -> Result<(), Error> {
    buffer.put_u16_be(bytes.len() as u16);
    buffer.put_slice(bytes);
    Ok(())
}

pub(crate) fn write_string(string: &str, buffer: &mut BytesMut) -> Result<(), Error> {
    write_bytes(string.as_bytes(), buffer)
}
