use std::io::Read;

/// Check if the packet is decodable from read_strem or not.
///
/// The main purpose of this function is to allow users to explicitly check if the packet
/// is available in the read stream like TCP socket or not.
pub fn check<R: Read>(read_stream: R) -> bool {
    unimplemented!("todo");
}
