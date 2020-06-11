use crate::*;
use bytes::Buf;

/// Decode bytes from a [BytesMut] buffer as a [Packet] enum.
///
/// The buf is never actually written to, it only takes a `BytesMut` instead of a `Bytes` to
/// allow using the same buffer to read bytes from network.
///
/// ```
/// # use mqttrs::*;
/// # use bytes::*;
/// // Fill a buffer with encoded data (probably from a `TcpStream`).
/// let mut buf = BytesMut::from(&[0b00110000, 11,
///                                0, 4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8,
///                                'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8] as &[u8]);
///
/// // Parse the bytes and check the result.
/// match decode(&mut buf) {
///     Ok(Some(Packet::Publish(p))) => {
///         assert_eq!(p.payload, "hello".as_bytes().to_vec());
///     },
///     // In real code you probably don't want to panic like that ;)
///     Ok(None) => panic!("not enough data"),
///     other => panic!("unexpected {:?}", other),
/// }
/// ```
///
/// [Packet]: ../enum.Packet.html
/// [BytesMut]: https://docs.rs/bytes/0.5.3/bytes/struct.BytesMut.html
// pub fn decode<'a>(mut buf: impl Buf) -> Result<Option<Packet<'a>>, Error> {
//     let mem = alloc::vec::Vec::with_capacity(1024);
//     decode_slice(&mem)
// }

pub fn clone_packet<'a, 'b>(
    mut input: impl Buf,
    output: &'b mut [u8],
) -> Result<Option<usize>, Error> {
    let mut offset = 0;
    while Header::new(input.bytes()[offset]).is_err() {
        offset += 1;
        if offset == input.remaining() {
            return Ok(None);
        }
    }

    let start = offset;
    if let Some((_, remaining_len)) = read_header(input.bytes(), &mut offset)? {
        let end = offset + remaining_len;
        output[..end - start].copy_from_slice(&input.bytes()[start..end]);
        input.advance(end - start);
        Ok(Some(end - start))
    } else {
        // Don't have a full packet
        Ok(None)
    }
}

pub fn decode_slice<'a>(buf: &'a [u8]) -> Result<Option<Packet<'a>>, Error> {
    let mut offset = 0;
    if let Some((header, remaining_len)) = read_header(buf, &mut offset)? {
        let r = read_packet(header, remaining_len, buf, &mut offset)?;
        Ok(Some(r))
    } else {
        // Don't have a full packet
        Ok(None)
    }
}

fn read_packet<'a>(
    header: Header,
    remaining_len: usize,
    buf: &'a [u8],
    offset: &mut usize,
) -> Result<Packet<'a>, Error> {
    Ok(match header.typ {
        PacketType::Pingreq => Packet::Pingreq,
        PacketType::Pingresp => Packet::Pingresp,
        PacketType::Disconnect => Packet::Disconnect,
        PacketType::Connect => Connect::from_buffer(buf, offset)?.into(),
        PacketType::Connack => Connack::from_buffer(buf, offset)?.into(),
        PacketType::Publish => Publish::from_buffer(&header, remaining_len, buf, offset)?.into(),
        PacketType::Puback => Packet::Puback(Pid::from_buffer(buf, offset)?),
        PacketType::Pubrec => Packet::Pubrec(Pid::from_buffer(buf, offset)?),
        PacketType::Pubrel => Packet::Pubrel(Pid::from_buffer(buf, offset)?),
        PacketType::Pubcomp => Packet::Pubcomp(Pid::from_buffer(buf, offset)?),
        PacketType::Subscribe => Subscribe::from_buffer(remaining_len, buf, offset)?.into(),
        PacketType::Suback => Suback::from_buffer(remaining_len, buf, offset)?.into(),
        PacketType::Unsubscribe => Unsubscribe::from_buffer(remaining_len, buf, offset)?.into(),
        PacketType::Unsuback => Packet::Unsuback(Pid::from_buffer(buf, offset)?),
    })
}

/// Read the parsed header and remaining_len from the buffer. Only return Some() and advance the
/// buffer position if there is enough data in the buffer to read the full packet.
pub(crate) fn read_header<'a>(
    buf: &'a [u8],
    offset: &mut usize,
) -> Result<Option<(Header, usize)>, Error> {
    let mut len: usize = 0;
    for pos in 0..=3 {
        if buf.len() > *offset + pos + 1 {
            let byte = buf[*offset + pos + 1];
            len += (byte as usize & 0x7F) << (pos * 7);
            if (byte & 0x80) == 0 {
                // Continuation bit == 0, length is parsed
                if buf.len() < *offset + 2 + pos + len {
                    // Won't be able to read full packet
                    return Ok(None);
                }
                // Parse header byte, skip past the header, and return
                let header = Header::new(buf[*offset])?;
                *offset += pos + 2;
                return Ok(Some((header, len)));
            }
        } else {
            // Couldn't read full length
            return Ok(None);
        }
    }
    // Continuation byte == 1 four times, that's illegal.
    Err(Error::InvalidHeader)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Header {
    pub typ: PacketType,
    pub dup: bool,
    pub qos: QoS,
    pub retain: bool,
}
impl Header {
    pub fn new(hd: u8) -> Result<Header, Error> {
        let (typ, flags_ok) = match hd >> 4 {
            1 => (PacketType::Connect, hd & 0b1111 == 0),
            2 => (PacketType::Connack, hd & 0b1111 == 0),
            3 => (PacketType::Publish, true),
            4 => (PacketType::Puback, hd & 0b1111 == 0),
            5 => (PacketType::Pubrec, hd & 0b1111 == 0),
            6 => (PacketType::Pubrel, hd & 0b1111 == 0b0010),
            7 => (PacketType::Pubcomp, hd & 0b1111 == 0),
            8 => (PacketType::Subscribe, hd & 0b1111 == 0b0010),
            9 => (PacketType::Suback, hd & 0b1111 == 0),
            10 => (PacketType::Unsubscribe, hd & 0b1111 == 0b0010),
            11 => (PacketType::Unsuback, hd & 0b1111 == 0),
            12 => (PacketType::Pingreq, hd & 0b1111 == 0),
            13 => (PacketType::Pingresp, hd & 0b1111 == 0),
            14 => (PacketType::Disconnect, hd & 0b1111 == 0),
            _ => (PacketType::Connect, false),
        };
        if !flags_ok {
            return Err(Error::InvalidHeader);
        }
        Ok(Header {
            typ,
            dup: hd & 0b1000 != 0,
            qos: QoS::from_u8((hd & 0b110) >> 1)?,
            retain: hd & 1 == 1,
        })
    }
}

pub(crate) fn read_str<'a>(buf: &'a [u8], offset: &mut usize) -> Result<&'a str, Error> {
    core::str::from_utf8(read_bytes(buf, offset)?).map_err(|e| Error::InvalidString(e))
}

pub(crate) fn read_bytes<'a>(buf: &'a [u8], offset: &mut usize) -> Result<&'a [u8], Error> {
    let len = ((buf[*offset] as usize) << 8) | buf[*offset + 1] as usize;
    *offset += 2;
    if len > buf[*offset..].len() {
        Err(Error::InvalidLength)
    } else {
        let bytes = &buf[*offset..*offset + len];
        *offset += len;
        Ok(bytes)
    }
}
