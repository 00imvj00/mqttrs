use crate::Packet;
use std::io;
use std::io::Write;

pub trait MqttWriter: Write {
    fn write_packet(&mut self, _packet: Packet) -> Result<(), io::Error> {
        //TODO: ENCODE PACKET TO BYTES ARRAY
        let data = [];
        let _ = self.write(&data);
        Ok(())
    }
}
