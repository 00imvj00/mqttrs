use crate::Packet;
use std::io::Read;

pub trait MqttReader: Read {
    //TODO: When read is successful, remove the bytes from self.
    fn read_packet(&mut self) -> Option<Packet> {
        None
    }
}
