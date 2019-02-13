use crate::MULTIPLIER;
use crate::{Header, Packet};

pub fn decode(buffer: &mut Vec<u8>) -> Option<Packet> {
    if let Some(header) = read_header(&buffer) {
        println!("Header read {:?}", header);
        None
    } else {
        None
    }
}

/* This will read the header of the stream */
fn read_header(buffer: &Vec<u8>) -> Option<Header> {
    if buffer.len() > 1 {
        let header_u8 = buffer.get(0).unwrap();
        if let Some(length) = read_length(buffer, 1) {
            let header = Header::new(*header_u8, length).unwrap();
            Some(header)
        } else {
            None
        }
    } else {
        None
    }
}

fn read_length(buffer: &Vec<u8>, mut pos: usize) -> Option<usize> {
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
    Some(len as usize)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_reader() {
        // let mut stream = vec![
        //     0x10, 39, 0x00, 0x04, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 0x04,
        //     0b11001110, // +username, +password, -will retain, will qos=1, +last_will, +clean_session
        //     0x00, 0x0a, // 10 sec
        //     0x00, 0x04, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, // client_id
        //     0x00, 0x02, '/' as u8, 'a' as u8, // will topic = '/a'
        //     0x00, 0x07, 'o' as u8, 'f' as u8, 'f' as u8, 'l' as u8, 'i' as u8, 'n' as u8,
        //     'e' as u8, // will msg = 'offline'
        //     0x00, 0x04, 'r' as u8, 'u' as u8, 's' as u8, 't' as u8, // username = 'rust'
        //     0x00, 0x02, 'm' as u8, 'q' as u8, // password = 'mq'
        // ];

        // assert_eq!(packet, None);
    }
}
