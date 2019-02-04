mod header;
mod packet;

pub use crate::{
    header::{Header, PacketType},
    packet::Packet,
};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
