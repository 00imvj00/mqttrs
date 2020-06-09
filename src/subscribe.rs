use crate::{decoder::*, encoder::*, *};
use bytes::BufMut;
#[cfg(feature = "derive")]
use serde::{Deserialize, Serialize};

/// Subscribe topic.
///
/// [Subscribe] packets contain a `Vec` of those.
///
/// [Subscribe]: struct.Subscribe.html
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "derive", derive(Serialize, Deserialize))]
pub struct SubscribeTopic<'a> {
    pub topic_path: &'a str,
    pub qos: QoS,
}

impl<'a> SubscribeTopic<'a> {
    pub(crate) fn from_buffer(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let topic_path = read_str(buf, offset)?;
        let qos = QoS::from_u8(buf[*offset])?;
        *offset +=1;
        Ok(SubscribeTopic { topic_path, qos })
    }
}

pub struct SubscribeTopicIter<'a> {
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for SubscribeTopicIter<'a> {
    type Item = SubscribeTopic<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        SubscribeTopic::from_buffer(self.buffer, &mut self.offset).ok()
    }
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
    pub(crate) fn from_buffer<'a>(buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let code = buf[*offset];
        *offset += 1;

        if code == 0x80 {
            Ok(SubscribeReturnCodes::Failure)
        } else {
            Ok(SubscribeReturnCodes::Success(QoS::from_u8(code)?))
        }
    }

    pub(crate) fn to_u8(&self) -> u8 {
        match *self {
            SubscribeReturnCodes::Failure => 0x80,
            SubscribeReturnCodes::Success(qos) => qos.to_u8(),
        }
    }
}

pub struct ReturnCodeIter<'a> {
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for ReturnCodeIter<'a> {
    type Item = SubscribeReturnCodes;

    fn next(&mut self) -> Option<Self::Item> {
        SubscribeReturnCodes::from_buffer(self.buffer, &mut self.offset).ok()
    }
}

/// Subscribe packet ([MQTT 3.8]).
///
/// [MQTT 3.8]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718063
#[derive(Debug, Clone, PartialEq)]
pub struct Subscribe<'a> {
    pub pid: Pid,
    topic_buf: &'a [u8],
}

/// Subsack packet ([MQTT 3.9]).
///
/// [MQTT 3.9]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718068
#[derive(Debug, Clone, PartialEq)]
pub struct Suback<'a> {
    pub pid: Pid,
    pub return_codes_buf: &'a [u8],
}

/// Unsubscribe packet ([MQTT 3.10]).
///
/// [MQTT 3.10]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718072
#[derive(Debug, Clone, PartialEq)]
pub struct Unsubscribe<'a> {
    pub pid: Pid,
    topic_buf: &'a [u8],
}

pub struct UnsubscribeIter<'a> {
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for UnsubscribeIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        read_str(self.buffer, &mut self.offset).ok()
    }
}

impl<'a> Subscribe<'a> {
    pub(crate) fn new(pid: Pid, topics: &'a [SubscribeTopic<'a>]) -> Self {
        Subscribe {
            pid,
            topic_buf: &[]
        }
    }

    pub(crate) fn from_buffer(remaining_len: usize, buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let payload_end = *offset + remaining_len;
        let pid = Pid::from_buffer(buf, offset)?;

        Ok(Subscribe { pid, topic_buf: &buf[*offset..payload_end] })
    }

    pub fn topics(&self) -> SubscribeTopicIter<'a> {
        SubscribeTopicIter {
            buffer: self.topic_buf,
            offset: 0
        }
    }

    pub(crate) fn to_buffer(&self, mut buf: impl BufMut) -> Result<usize, Error> {
        let header: u8 = 0b10000010;
        check_remaining(&mut buf, 1)?;
        buf.put_u8(header);

        // Length: pid(2) + topic.for_each(2+len + qos(1))
        let mut length = 2;
        for topic in self.topics() {
            length += topic.topic_path.len() + 2 + 1;
        }
        let write_len = write_length(length, &mut buf)? + 1;

        // Pid
        self.pid.to_buffer(&mut buf)?;

        // Topics
        for topic in self.topics() {
            write_string(topic.topic_path, &mut buf)?;
            buf.put_u8(topic.qos.to_u8());
        }

        Ok(write_len)
    }
}

impl<'a> Unsubscribe<'a> {
    pub(crate) fn new(pid: Pid, topics: &'a [&'a str]) -> Self {
        Unsubscribe {
            pid,
            topic_buf: &[]
        }
    }

    pub(crate) fn from_buffer(remaining_len: usize, buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let payload_end = *offset + remaining_len;
        let pid = Pid::from_buffer(buf, offset)?;

        Ok(Unsubscribe { pid, topic_buf: &buf[*offset..payload_end] })
    }

    pub fn topics(&self) -> UnsubscribeIter<'a> {
        UnsubscribeIter {
            buffer: self.topic_buf,
            offset: 0
        }
    }

    pub(crate) fn to_buffer(&self, mut buf: impl BufMut) -> Result<usize, Error> {
        let header: u8 = 0b10100010;
        let mut length = 2;
        for topic in self.topics() {
            length += 2 + topic.len();
        }
        check_remaining(&mut buf, 1)?;
        buf.put_u8(header);

        let write_len = write_length(length, &mut buf)? + 1;
        self.pid.to_buffer(&mut buf)?;
        for topic in self.topics() {
            write_string(topic, &mut buf)?;
        }
        Ok(write_len)
    }
}

impl<'a> Suback<'a> {
    pub(crate) fn new(pid: Pid, return_codes: &'a [SubscribeReturnCodes]) -> Self {
        Suback {
            pid,
            return_codes_buf: &[]
        }
    }

    pub(crate) fn from_buffer(remaining_len: usize, buf: &'a [u8], offset: &mut usize) -> Result<Self, Error> {
        let payload_end = *offset + remaining_len;
        let pid = Pid::from_buffer(buf, offset)?;
        Ok(Suback { pid, return_codes_buf: &buf[*offset..payload_end] })
    }

    pub fn return_codes(&self) -> ReturnCodeIter<'a> {
        ReturnCodeIter {
            buffer: self.return_codes_buf,
            offset: 0
        }
    }

    pub(crate) fn to_buffer(&self, mut buf: impl BufMut) -> Result<usize, Error> {
        let header: u8 = 0b10010000;
        let length = 2 + self.return_codes_buf.len();
        check_remaining(&mut buf, 1)?;
        buf.put_u8(header);

        let write_len = write_length(length, &mut buf)? + 1;
        self.pid.to_buffer(&mut buf)?;
        for rc in self.return_codes() {
            buf.put_u8(rc.to_u8());
        }
        Ok(write_len)
    }
}
