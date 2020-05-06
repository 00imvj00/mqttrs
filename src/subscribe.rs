use crate::{decoder::*, encoder::*, *};
use bytes::{Buf, BufMut};
#[cfg(feature = "derive")]
use serde::{Deserialize, Serialize};
use alloc::{string::String, vec::Vec};

/// Subscribe topic.
///
/// [Subscribe] packets contain a `Vec` of those.
///
/// [Subscribe]: struct.Subscribe.html
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "derive", derive(Serialize, Deserialize))]
pub struct SubscribeTopic {
    pub topic_path: String,
    pub qos: QoS,
}

/// Subscribe return value.
///
/// [Suback] packets contain a `Vec` of those.
///
/// [Suback]: struct.Subscribe.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscribeReturnCodes {
    Success(QoS),
    Failure,
}
impl SubscribeReturnCodes {
    pub(crate) fn to_u8(&self) -> u8 {
        match *self {
            SubscribeReturnCodes::Failure => 0x80,
            SubscribeReturnCodes::Success(qos) => qos.to_u8(),
        }
    }
}

/// Subscribe packet ([MQTT 3.8]).
///
/// [MQTT 3.8]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718063
#[derive(Debug, Clone, PartialEq)]
pub struct Subscribe {
    pub pid: Pid,
    pub topics: Vec<SubscribeTopic>,
}

/// Subsack packet ([MQTT 3.9]).
///
/// [MQTT 3.9]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718068
#[derive(Debug, Clone, PartialEq)]
pub struct Suback {
    pub pid: Pid,
    pub return_codes: Vec<SubscribeReturnCodes>,
}

/// Unsubscribe packet ([MQTT 3.10]).
///
/// [MQTT 3.10]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718072
#[derive(Debug, Clone, PartialEq)]
pub struct Unsubscribe {
    pub pid: Pid,
    pub topics: Vec<String>,
}

impl Subscribe {
    pub(crate) fn from_buffer(mut buf: impl Buf) -> Result<Self, Error> {
        let pid = Pid::from_buffer(&mut buf)?;
        let mut topics: Vec<SubscribeTopic> = Vec::new();
        while buf.remaining() != 0 {
            let topic_path = read_string(&mut buf)?;
            let qos = QoS::from_u8(buf.get_u8())?;
            let topic = SubscribeTopic { topic_path, qos };
            topics.push(topic);
        }
        Ok(Subscribe { pid, topics })
    }

    pub(crate) fn to_buffer(&self, mut buf: impl BufMut) -> Result<usize, Error> {
        let header: u8 = 0b10000010;
        check_remaining(&mut buf, 1)?;
        buf.put_u8(header);

        // Length: pid(2) + topic.for_each(2+len + qos(1))
        let mut length = 2;
        for topic in &self.topics {
            length += topic.topic_path.len() + 2 + 1;
        }
        let write_len = write_length(length, &mut buf)? + 1;

        // Pid
        self.pid.to_buffer(&mut buf)?;

        // Topics
        for topic in &self.topics {
            write_string(topic.topic_path.as_ref(), &mut buf)?;
            buf.put_u8(topic.qos.to_u8());
        }

        Ok(write_len)
    }
}

impl Unsubscribe {
    pub(crate) fn from_buffer(mut buf: impl Buf) -> Result<Self, Error> {
        let pid = Pid::from_buffer(&mut buf)?;
        let mut topics: Vec<String> = Vec::new();
        while buf.remaining() != 0 {
            let topic_path = read_string(&mut buf)?;
            topics.push(topic_path);
        }
        Ok(Unsubscribe { pid, topics })
    }

    pub(crate) fn to_buffer(&self, mut buf: impl BufMut) -> Result<usize, Error> {
        let header: u8 = 0b10100010;
        let mut length = 2;
        for topic in &self.topics {
            length += 2 + topic.len();
        }
        check_remaining(&mut buf, 1)?;
        buf.put_u8(header);

        let write_len = write_length(length, &mut buf)? + 1;
        self.pid.to_buffer(&mut buf)?;
        for topic in &self.topics {
            write_string(topic.as_ref(), &mut buf)?;
        }
        Ok(write_len)
    }
}

impl Suback {
    pub(crate) fn from_buffer(mut buf: impl Buf) -> Result<Self, Error> {
        let pid = Pid::from_buffer(&mut buf)?;
        let mut return_codes: Vec<SubscribeReturnCodes> = Vec::new();
        while buf.remaining() != 0 {
            let code = buf.get_u8();
            let r = if code == 0x80 {
                SubscribeReturnCodes::Failure
            } else {
                SubscribeReturnCodes::Success(QoS::from_u8(code)?)
            };
            return_codes.push(r);
        }
        Ok(Suback { return_codes, pid })
    }
    pub(crate) fn to_buffer(&self, mut buf: impl BufMut) -> Result<usize, Error> {
        let header: u8 = 0b10010000;
        let length = 2 + self.return_codes.len();
        check_remaining(&mut buf, 1)?;
        buf.put_u8(header);

        let write_len = write_length(length, &mut buf)? + 1;
        self.pid.to_buffer(&mut buf)?;
        for rc in &self.return_codes {
            buf.put_u8(rc.to_u8());
        }
        Ok(write_len)
    }
}
