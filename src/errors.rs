use std::fmt;
use std::io::ErrorKind;
/// Errors returned by [`encode()`] and [`decode()`].
///
/// [`encode()`]: fn.encode.html
/// [`decode()`]: fn.decode.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Not enough space in the write buffer.
    ///
    /// It is the caller's responsiblity to pass a big enough buffer to `encode()`.
    WriteZero,
    /// Tried to encode or decode a ProcessIdentifier==0.
    InvalidPid,
    /// Tried to decode a QoS > 2.
    InvalidQos(u8),
    /// Tried to decode a ConnectReturnCode > 5.
    InvalidConnectReturnCode(u8),
    /// Tried to decode an unknown protocol.
    #[cfg(feature = "std")]
    InvalidProtocol(std::string::String, u8),
    #[cfg(not(feature = "std"))]
    InvalidProtocol(heapless::String<heapless::consts::U10>, u8),
    /// Tried to decode an invalid fixed header (packet type, flags, or remaining_length).
    InvalidHeader,
    /// Trying to encode/decode an invalid length.
    ///
    /// The difference with `WriteZero`/`UnexpectedEof` is that it refers to an invalid/corrupt
    /// length rather than a buffer size issue.
    InvalidLength,
    /// Trying to decode a non-utf8 string.
    InvalidString(core::str::Utf8Error),
    /// Catch-all error when converting from `std::io::Error`.
    ///
    /// Note: Only available when std is available.
    /// You'll hopefully never see this.
    #[cfg(feature = "std")]
    IoError(ErrorKind, std::string::String),
}

#[cfg(feature = "std")]
//impl ErrorTrait for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//#[cfg(feature = "std")]
//impl From<Error> for IoError {
//fn from(err: Error) -> IoError {
//match err {
//Error::WriteZero => IoError::new(ErrorKind::WriteZero, err),
//_ => IoError::new(ErrorKind::InvalidData, err),
//}
//}
//}

//#[cfg(feature = "std")]
//impl From<IoError> for Error {
//fn from(err: IoError) -> Error {
//match err.kind() {
//ErrorKind::WriteZero => Error::WriteZero,
//k => Error::IoError(k, format!("{}", err)),
//}
//}
//}
