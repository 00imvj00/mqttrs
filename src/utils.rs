#[cfg(feature = "derive")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use crate::errors::Error;

use core::num::NonZeroU16;
use std::convert::TryFrom;

/// Packet Identifier.
///
/// For packets with [`QoS::AtLeastOne` or `QoS::ExactlyOnce`] delivery.
///
/// ```rust
/// # use mqttrs::{Packet, Pid, QosPid};
/// # use std::convert::TryFrom;
/// #[derive(Default)]
/// struct Session {
///    pid: Pid,
/// }
/// impl Session {
///    pub fn next_pid(&mut self) -> Pid {
///        self.pid = self.pid + 1;
///        self.pid
///    }
/// }
///
/// let mut sess = Session::default();
/// assert_eq!(2, sess.next_pid().get());
/// assert_eq!(Pid::try_from(3).unwrap(), sess.next_pid());
/// ```
///
/// The spec ([MQTT-2.3.1-1], [MQTT-2.2.1-3]) disallows a pid of 0.
///
/// [`QoS::AtLeastOne` or `QoS::ExactlyOnce`]: enum.QoS.html
/// [MQTT-2.3.1-1]: https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718025
/// [MQTT-2.2.1-3]: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901026
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "derive", derive(Serialize, Deserialize))]
pub struct Pid(NonZeroU16);
impl Pid {
    /// Returns a new `Pid` with value `1`.
    pub fn new() -> Self {
        Pid(NonZeroU16::new(1).unwrap())
    }

    /// Get the `Pid` as a raw `u16`.
    pub fn get(self) -> u16 {
        self.0.get()
    }

    pub(crate) fn from_buffer(buf: &[u8], offset: &mut usize) -> Result<Self, Error> {
        let pid = ((buf[*offset] as u16) << 8) | buf[*offset + 1] as u16;
        *offset += 2;
        Self::try_from(pid)
    }

    pub(crate) fn to_buffer(self, buf: &mut [u8], offset: &mut usize) -> Result<(), Error> {
        write_u16(buf, offset, self.get())
    }
}

impl Default for Pid {
    fn default() -> Pid {
        Pid::new()
    }
}

impl core::ops::Add<u16> for Pid {
    type Output = Pid;

    /// Adding a `u16` to a `Pid` will wrap around and avoid 0.
    fn add(self, u: u16) -> Pid {
        let n = match self.get().overflowing_add(u) {
            (n, false) => n,
            (n, true) => n + 1,
        };
        Pid(NonZeroU16::new(n).unwrap())
    }
}

impl core::ops::Sub<u16> for Pid {
    type Output = Pid;

    /// Adding a `u16` to a `Pid` will wrap around and avoid 0.
    fn sub(self, u: u16) -> Pid {
        let n = match self.get().overflowing_sub(u) {
            (0, _) => core::u16::MAX,
            (n, false) => n,
            (n, true) => n - 1,
        };
        Pid(NonZeroU16::new(n).unwrap())
    }
}

impl From<Pid> for u16 {
    /// Convert `Pid` to `u16`.
    fn from(p: Pid) -> Self {
        p.0.get()
    }
}

impl TryFrom<u16> for Pid {
    type Error = Error;

    /// Convert `u16` to `Pid`. Will fail for value 0.
    fn try_from(u: u16) -> Result<Self, Error> {
        match NonZeroU16::new(u) {
            Some(nz) => Ok(Pid(nz)),
            None => Err(Error::InvalidPid),
        }
    }
}

pub(crate) fn write_u8(buf: &mut [u8], offset: &mut usize, val: u8) -> Result<(), Error> {
    buf[*offset] = val;
    *offset += 1;
    Ok(())
}

pub(crate) fn write_u16(buf: &mut [u8], offset: &mut usize, val: u16) -> Result<(), Error> {
    write_u8(buf, offset, (val >> 8) as u8)?;
    write_u8(buf, offset, (val & 0xFF) as u8)
}

pub(crate) fn write_bytes(buf: &mut [u8], offset: &mut usize, bytes: &[u8]) -> Result<(), Error> {
    write_u16(buf, offset, bytes.len() as u16)?;

    for &byte in bytes {
        write_u8(buf, offset, byte)?;
    }
    Ok(())
}

pub(crate) fn write_string(buf: &mut [u8], offset: &mut usize, string: &str) -> Result<(), Error> {
    write_bytes(buf, offset, string.as_bytes())
}

#[cfg(test)]
mod test {
    use crate::Pid;
    use core::convert::TryFrom;
    use std::vec;

    #[test]
    fn pid_add_sub() {
        let t: Vec<(u16, u16, u16, u16)> = vec![
            (2, 1, 1, 3),
            (100, 1, 99, 101),
            (1, 1, core::u16::MAX, 2),
            (1, 2, core::u16::MAX - 1, 3),
            (1, 3, core::u16::MAX - 2, 4),
            (core::u16::MAX, 1, core::u16::MAX - 1, 1),
            (core::u16::MAX, 2, core::u16::MAX - 2, 2),
            (10, core::u16::MAX, 10, 10),
            (10, 0, 10, 10),
            (1, 0, 1, 1),
            (core::u16::MAX, 0, core::u16::MAX, core::u16::MAX),
        ];
        for (cur, d, prev, next) in t {
            let sub = Pid::try_from(cur).unwrap() - d;
            let add = Pid::try_from(cur).unwrap() + d;
            assert_eq!(prev, sub.get(), "{} - {} should be {}", cur, d, prev);
            assert_eq!(next, add.get(), "{} + {} should be {}", cur, d, next);
        }
    }
}
