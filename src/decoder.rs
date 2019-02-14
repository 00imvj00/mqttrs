use crate::MULTIPLIER;
use crate::*;

#[allow(dead_code)]
pub fn decode(mut buffer: Vec<u8>) -> Option<Packet> {
    if let Some((header, header_size)) = read_header(&buffer) {
        buffer = buffer.split_off(header_size); //removing header bytes, possible ALLOC
        if header.len() == 0 {
            let p = match header.packet() {
                PacketType::PingReq => Packet::PingReq,
                PacketType::PingResp => Packet::PingResp,
                PacketType::Disconnect => Packet::Disconnect,
                _ => {
                    println!("Phantom Packet. Error ");
                    Packet::None
                }
            };
            Some(p)
        } else if buffer.len() >= header.len() {
            let remaining = buffer.split_off(header.len());
            let p = read_packet(header.packet(), buffer);
            buffer = remaining;
            Some(p)
        } else {
            None
        }
    } else {
        None
    }
}

fn read_packet(t: PacketType, buffer: Vec<u8>) -> Packet {
    match t {
        PacketType::Connect => Packet::None,
        PacketType::Connack => Packet::None,
        PacketType::Publish => Packet::None,
        PacketType::Puback => Packet::None,
        PacketType::Pubrec => Packet::None,
        PacketType::Pubrel => Packet::None,
        PacketType::PubComp => Packet::None,
        PacketType::Subscribe => Packet::None,
        PacketType::SubAck => Packet::None,
        PacketType::UnSubscribe => Packet::None,
        PacketType::UnSubAck => Packet::None,
        _ => {
            println!("Phantom Packet. Error ");
            Packet::None
        }
    }
}
/* This will read the header of the stream */
fn read_header(buffer: &Vec<u8>) -> Option<(Header, usize)> {
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

fn read_length(buffer: &Vec<u8>, mut pos: usize) -> Option<(usize, usize)> {
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
    Some((len as usize, mult))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_connect() {
        let mut data = vec![
            0b00010000, 39, 0x00, 0x04, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 0x04,
            0b11001110, // +username, +password, -will retain, will qos=1, +last_will, +clean_session
            0x00, 0x0a, // 10 sec
            0x00, 0x04, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, // client_id
            0x00, 0x02, '/' as u8, 'a' as u8, // will topic = '/a'
            0x00, 0x07, 'o' as u8, 'f' as u8, 'f' as u8, 'l' as u8, 'i' as u8, 'n' as u8,
            'e' as u8, // will msg = 'offline'
            0x00, 0x04, 'r' as u8, 'u' as u8, 's' as u8, 't' as u8, // username = 'rust'
            0x00, 0x02, 'm' as u8, 'q' as u8, // password = 'mq'
        ];
        let d = super::decode(data);
        assert_eq!(d, None);
        // assert_eq!(packet, None);
    }
}
