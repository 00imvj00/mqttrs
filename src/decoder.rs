use crate::{header::Header, *};
use bytes::{Buf, BytesMut, IntoBuf};
use std::io::{Error, ErrorKind};

/// Decode network bytes into a [Packet] enum.
///
/// [Packet]: ../enum.Packet.html
pub fn decode(buffer: &mut BytesMut) -> Result<Option<Packet>, Error> {
    if let Some((header, header_size)) = read_header(buffer) {
        if buffer.len() >= header.len() + header_size {
            //NOTE: Check if buffer has, header bytes + remaining length bytes in buffer.
            buffer.split_to(header_size); //NOTE: Remove header bytes from buffer.
            let p = read_packet(header, buffer)?; //NOTE: Read remaining packet.
            Ok(Some(p))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn read_packet(header: Header, buffer: &mut BytesMut) -> Result<Packet, Error> {
    Ok(match header.packet() {
        PacketType::PingReq => Packet::PingReq,
        PacketType::PingResp => Packet::PingResp,
        PacketType::Disconnect => Packet::Disconnect,
        PacketType::Connect => {
            Packet::Connect(Connect::from_buffer(&mut buffer.split_to(header.len()))?)
        }
        PacketType::Connack => {
            Packet::Connack(Connack::from_buffer(&mut buffer.split_to(header.len()))?)
        }
        PacketType::Publish => Packet::Publish(Publish::from_buffer(
            &header,
            &mut buffer.split_to(header.len()),
        )?),
        PacketType::Puback => Packet::Puback(PacketIdentifier::from_buffer(buffer)?),
        PacketType::Pubrec => Packet::Pubrec(PacketIdentifier::from_buffer(buffer)?),
        PacketType::Pubrel => Packet::Pubrel(PacketIdentifier::from_buffer(buffer)?),
        PacketType::PubComp => Packet::PubComp(PacketIdentifier::from_buffer(buffer)?),
        PacketType::Subscribe => {
            Packet::Subscribe(Subscribe::from_buffer(&mut buffer.split_to(header.len()))?)
        }
        PacketType::SubAck => {
            Packet::SubAck(Suback::from_buffer(&mut buffer.split_to(header.len()))?)
        }
        PacketType::UnSubscribe => Packet::UnSubscribe(Unsubscribe::from_buffer(
            &mut buffer.split_to(header.len()),
        )?),
        PacketType::UnSubAck => Packet::UnSubAck(PacketIdentifier::from_buffer(buffer)?),
    })
}

/// Read the header of the stream
fn read_header(buffer: &mut BytesMut) -> Option<(Header, usize)> {
    if buffer.len() > 1 {
        let header_u8 = buffer.get(0).unwrap();
        if let Some((length, size)) = read_length(buffer, 1) {
            let header = Header::new(*header_u8, length).unwrap();
            Some((header, size + 1))
        } else {
            None
        }
    } else {
        None
    }
}

fn read_length(buffer: &BytesMut, mut pos: usize) -> Option<(usize, usize)> {
    let mut mult: usize = 1;
    let mut len: usize = 0;
    let mut done = false;

    while !done {
        let byte = (*buffer.get(pos).unwrap()) as usize;
        len += (byte & 0x7F) * mult;
        mult *= 0x80;
        if mult > MULTIPLIER {
            return None;
        }
        if (byte & 0x80) == 0 {
            done = true;
        } else {
            pos += 1;
        }
    }
    Some((len as usize, pos))
}

pub(crate) fn read_string(buffer: &mut BytesMut) -> Result<String, Error> {
    String::from_utf8(read_bytes(buffer)?)
        .map_err(|_| Error::new(ErrorKind::InvalidData, "Non-utf8 string"))
}

pub(crate) fn read_bytes(buffer: &mut BytesMut) -> Result<Vec<u8>, Error> {
    let length = buffer.split_to(2).into_buf().get_u16_be();
    if length as usize > buffer.len() {
        Err(Error::new(ErrorKind::InvalidData, "length > buffer.len()"))
    } else {
        Ok(buffer.split_to(length as usize).to_vec())
    }
}


#[cfg(test)]
mod test {
    use crate::decode;
    use bytes::BytesMut;
    use std::io::ErrorKind;

    #[test]
    fn non_utf8_string() {
        let mut data = BytesMut::from(vec![
            0b00110000, 10, // type=Publish, remaining_len=10
            0x00, 0x03, 'a' as u8, '/' as u8, 0xc0 as u8, // Topic with Invalid utf8
            'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, // payload
        ]);
        assert_eq!(
            ErrorKind::InvalidData,
            decode(&mut data).unwrap_err().kind()
        );
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
        assert_eq!(
            ErrorKind::InvalidData,
            decode(&mut data).unwrap_err().kind()
        );
    }
}
