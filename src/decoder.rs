use crate::{header::Header, *};
use bytes::{Buf, BytesMut, IntoBuf};

/// Decode network bytes into a [Packet] enum.
///
/// [Packet]: ../enum.Packet.html
pub fn decode(buffer: &mut BytesMut) -> Result<Option<Packet>, Error> {
    if let Some((header, remaining_len)) = read_header(buffer)? {
        // Advance the buffer position to the next packet, and parse the current packet
        let p = &mut buffer.split_to(remaining_len);
        Ok(Some(read_packet(header, p)?))
    } else {
        // Don't have a full packet
        Ok(None)
    }
}

fn read_packet(header: Header, buffer: &mut BytesMut) -> Result<Packet, Error> {
    Ok(match header.typ {
        PacketType::Pingreq => Packet::Pingreq,
        PacketType::Pingresp => Packet::Pingresp,
        PacketType::Disconnect => Packet::Disconnect,
        PacketType::Connect => Connect::from_buffer(buffer)?.into(),
        PacketType::Connack => Connack::from_buffer(buffer)?.into(),
        PacketType::Publish => Publish::from_buffer(&header, buffer)?.into(),
        PacketType::Puback => Packet::Puback(Pid::from_buffer(buffer)?),
        PacketType::Pubrec => Packet::Pubrec(Pid::from_buffer(buffer)?),
        PacketType::Pubrel => Packet::Pubrel(Pid::from_buffer(buffer)?),
        PacketType::Pubcomp => Packet::Pubcomp(Pid::from_buffer(buffer)?),
        PacketType::Subscribe => Subscribe::from_buffer(buffer)?.into(),
        PacketType::Suback => Suback::from_buffer(buffer)?.into(),
        PacketType::Unsubscribe => Unsubscribe::from_buffer(buffer)?.into(),
        PacketType::Unsuback => Packet::Unsuback(Pid::from_buffer(buffer)?),
    })
}

/// Read the parsed header and remaining_len from the buffer. Only return Some() and advance the
/// buffer position if there is enough data in th ebuffer to read the full packet.
fn read_header(buffer: &mut BytesMut) -> Result<Option<(Header, usize)>, Error> {
    let mut len: usize = 0;
    for pos in 0..=3 {
        if let Some(&byte) = buffer.get(pos + 1) {
            len += (byte as usize & 0x7F) << (pos * 7);
            if (byte & 0x80) == 0 {
                // Continuation bit == 0, length is parsed
                if buffer.len() < 2 + pos + len {
                    // Won't be able to read full packet
                    return Ok(None);
                }
                // Parse header byte, skip past the header, and return
                let header = Header::new(*buffer.get(0).unwrap())?;
                buffer.advance(pos + 2);
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

pub(crate) fn read_string(buffer: &mut BytesMut) -> Result<String, Error> {
    String::from_utf8(read_bytes(buffer)?).map_err(|e| Error::InvalidString(e.utf8_error()))
}

pub(crate) fn read_bytes(buffer: &mut BytesMut) -> Result<Vec<u8>, Error> {
    let len = buffer.split_to(2).into_buf().get_u16_be() as usize;
    if len > buffer.len() {
        Err(Error::InvalidLength)
    } else {
        Ok(buffer.split_to(len).to_vec())
    }
}

#[cfg(test)]
mod test {
    use crate::{decoder::read_header, header::Header, *};
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
            let buf = &mut BytesMut::from(vec![n, 0]);
            assert_eq!(res, read_header(buf), "{:08b}", n);
        }
    }

    /// Test decoding of length and actual buffer len.
    #[rustfmt::skip]
    #[test]
    fn header_len() {
        let h = header!(Connect, false, AtMostOnce, false);
        for (res, bytes, buflen) in vec![
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
            let mut buf = BytesMut::from(bytes);
            buf.resize(buflen, 0);
            assert_eq!(res, read_header(&mut buf));
        }
    }

    #[test]
    fn non_utf8_string() {
        let mut data = BytesMut::from(vec![
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
        let mut data = BytesMut::from(vec![
            0b00010000, 20, // Connect packet, remaining_len=20
            0x00, 0x04, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 0x04,
            0b01000000, // +password
            0x00, 0x0a, // keepalive 10 sec
            0x00, 0x04, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, // client_id
            0x00, 0x03, 'm' as u8, 'q' as u8, // password with invalid length
        ]);
        assert_eq!(Err(Error::InvalidLength), decode(&mut data));
    }
}
