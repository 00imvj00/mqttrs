use crate::*;
use alloc::{string::String, vec::Vec};
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
pub fn decode(mut buf: impl Buf) -> Result<Option<Packet>, Error> {
    if let Some((header, remaining_len)) = read_header(&mut buf)? {
        // Advance the buffer position to the next packet, and parse the current packet
        let r = read_packet(header, &mut &buf.bytes()[..remaining_len]);
        buf.advance(remaining_len);
        // Make sure to advance the buffer, before checking the result of read_packet
        Ok(Some(r?))
    } else {
        // Don't have a full packet
        Ok(None)
    }
}

fn read_packet(header: Header, buf: impl Buf) -> Result<Packet, Error> {
    Ok(match header.typ {
        PacketType::Pingreq => Packet::Pingreq,
        PacketType::Pingresp => Packet::Pingresp,
        PacketType::Disconnect => Packet::Disconnect,
        PacketType::Connect => Connect::from_buffer(buf)?.into(),
        PacketType::Connack => Connack::from_buffer(buf)?.into(),
        PacketType::Publish => Publish::from_buffer(&header, buf)?.into(),
        PacketType::Puback => Packet::Puback(Pid::from_buffer(buf)?),
        PacketType::Pubrec => Packet::Pubrec(Pid::from_buffer(buf)?),
        PacketType::Pubrel => Packet::Pubrel(Pid::from_buffer(buf)?),
        PacketType::Pubcomp => Packet::Pubcomp(Pid::from_buffer(buf)?),
        PacketType::Subscribe => Subscribe::from_buffer(buf)?.into(),
        PacketType::Suback => Suback::from_buffer(buf)?.into(),
        PacketType::Unsubscribe => Unsubscribe::from_buffer(buf)?.into(),
        PacketType::Unsuback => Packet::Unsuback(Pid::from_buffer(buf)?),
    })
}

/// Read the parsed header and remaining_len from the buffer. Only return Some() and advance the
/// buffer position if there is enough data in the buffer to read the full packet.
fn read_header(mut buf: impl Buf) -> Result<Option<(Header, usize)>, Error> {
    let mut len: usize = 0;
    for pos in 0..=3 {
        if buf.remaining() > pos + 1 {
            let byte = buf.bytes()[pos + 1];
            len += (byte as usize & 0x7F) << (pos * 7);
            if (byte & 0x80) == 0 {
                // Continuation bit == 0, length is parsed
                if buf.remaining() < 2 + pos + len {
                    // Won't be able to read full packet
                    return Ok(None);
                }
                // Parse header byte, skip past the header, and return
                let header = Header::new(buf.get_u8())?;
                buf.advance(pos + 1);
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

pub(crate) fn read_string(buf: impl Buf) -> Result<String, Error> {
    String::from_utf8(read_bytes(buf)?).map_err(|e| Error::InvalidString(e.utf8_error()))
}

pub(crate) fn read_bytes(mut buf: impl Buf) -> Result<Vec<u8>, Error> {
    let len = buf.get_u16() as usize;
    if len > buf.remaining() {
        Err(Error::InvalidLength)
    } else {
        let r = buf.bytes()[..len].to_vec();
        buf.advance(len);
        Ok(r)
    }
}

#[cfg(test)]
mod test {
    use crate::decoder::*;
    use alloc::vec;
    use bytes::BytesMut;

    macro_rules! header {
        ($t:ident, $d:expr, $q:ident, $r:expr) => {
            Header {
                typ: PacketType::$t,
                dup: $d,
                qos: QoS::$q,
                retain: $r,
            }
        };
    }

    fn bm(d: &[u8]) -> BytesMut {
        BytesMut::from(d)
    }

    /// Test all possible header first byte, using remaining_len=0.
    #[test]
    fn header_firstbyte() {
        let valid = vec![
            (0b0001_0000, header!(Connect, false, AtMostOnce, false)),
            (0b0010_0000, header!(Connack, false, AtMostOnce, false)),
            (0b0011_0000, header!(Publish, false, AtMostOnce, false)),
            (0b0011_0001, header!(Publish, false, AtMostOnce, true)),
            (0b0011_0010, header!(Publish, false, AtLeastOnce, false)),
            (0b0011_0011, header!(Publish, false, AtLeastOnce, true)),
            (0b0011_0100, header!(Publish, false, ExactlyOnce, false)),
            (0b0011_0101, header!(Publish, false, ExactlyOnce, true)),
            (0b0011_1000, header!(Publish, true, AtMostOnce, false)),
            (0b0011_1001, header!(Publish, true, AtMostOnce, true)),
            (0b0011_1010, header!(Publish, true, AtLeastOnce, false)),
            (0b0011_1011, header!(Publish, true, AtLeastOnce, true)),
            (0b0011_1100, header!(Publish, true, ExactlyOnce, false)),
            (0b0011_1101, header!(Publish, true, ExactlyOnce, true)),
            (0b0100_0000, header!(Puback, false, AtMostOnce, false)),
            (0b0101_0000, header!(Pubrec, false, AtMostOnce, false)),
            (0b0110_0010, header!(Pubrel, false, AtLeastOnce, false)),
            (0b0111_0000, header!(Pubcomp, false, AtMostOnce, false)),
            (0b1000_0010, header!(Subscribe, false, AtLeastOnce, false)),
            (0b1001_0000, header!(Suback, false, AtMostOnce, false)),
            (0b1010_0010, header!(Unsubscribe, false, AtLeastOnce, false)),
            (0b1011_0000, header!(Unsuback, false, AtMostOnce, false)),
            (0b1100_0000, header!(Pingreq, false, AtMostOnce, false)),
            (0b1101_0000, header!(Pingresp, false, AtMostOnce, false)),
            (0b1110_0000, header!(Disconnect, false, AtMostOnce, false)),
        ];
        for n in 0..=255 {
            let res = match valid.iter().find(|(byte, _)| *byte == n) {
                Some((_, header)) => Ok(Some((*header, 0))),
                None if ((n & 0b110) == 0b110) && (n >> 4 == 3) => Err(Error::InvalidQos(3)),
                None => Err(Error::InvalidHeader),
            };
            let mut buf = bm(&[n, 0]);
            assert_eq!(res, read_header(&mut buf), "{:08b}", n);
        }
    }

    /// Test decoding of length and actual buffer len.
    #[rustfmt::skip]
    #[test]
    fn header_len() {
        let h = header!(Connect, false, AtMostOnce, false);
        for (res, mut bytes, buflen) in vec![
            (Ok(Some((h, 0))),          vec![1 << 4, 0],   2),
            (Ok(None),                  vec![1 << 4, 127], 128),
            (Ok(Some((h, 127))),        vec![1 << 4, 127], 129),
            (Ok(None),                  vec![1 << 4, 0x80], 2),
            (Ok(Some((h, 0))),          vec![1 << 4, 0x80, 0], 3), //Weird encoding for "0" buf matches spec
            (Ok(Some((h, 128))),        vec![1 << 4, 0x80, 1], 131),
            (Ok(None),                  vec![1 << 4, 0x80+16, 78], 10002),
            (Ok(Some((h, 10000))),      vec![1 << 4, 0x80+16, 78], 10003),
            (Err(Error::InvalidHeader), vec![1 << 4, 0x80, 0x80, 0x80, 0x80], 10),
        ] {
            bytes.resize(buflen, 0);
            let mut buf = bm(bytes.as_slice());
            assert_eq!(res, read_header(&mut buf));
        }
    }

    #[test]
    fn non_utf8_string() {
        let mut data = bm(&[
            0b00110000, 10, // type=Publish, remaining_len=10
            0x00, 0x03, 'a' as u8, '/' as u8, 0xc0 as u8, // Topic with Invalid utf8
            'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, // payload
        ]);
        assert!(match decode(&mut data) {
            Err(Error::InvalidString(_)) => true,
            _ => false,
        });
    }

    /// Validity of remaining_len is tested exhaustively elsewhere, this is for inner lengths, which
    /// are rarer.
    #[test]
    fn inner_length_too_long() {
        let mut data = bm(&[
            0b00010000, 20, // Connect packet, remaining_len=20
            0x00, 0x04, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 0x04,
            0b01000000, // +password
            0x00, 0x0a, // keepalive 10 sec
            0x00, 0x04, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, // client_id
            0x00, 0x03, 'm' as u8, 'q' as u8, // password with invalid length
        ]);
        assert_eq!(Err(Error::InvalidLength), decode(&mut data));

        let mut slice = &[
            0b00010000, 20, // Connect packet, remaining_len=20
            0x00, 0x04, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 0x04,
            0b01000000, // +password
            0x00, 0x0a, // keepalive 10 sec
            0x00, 0x04, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, // client_id
            0x00, 0x03, 'm' as u8, 'q' as u8, // password with invalid length
        ][..];

        assert_eq!(Err(Error::InvalidLength), decode(&mut slice));
        assert_eq!(slice[..], []);

    }
}
