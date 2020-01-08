use crate::{decoder::*, encoder::*, *};
use bytes::{Buf, BufMut, BytesMut};
#[cfg(feature = "derive")]
use serde::{Deserialize, Serialize};

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
    pub(crate) fn from_buffer(buf: &mut BytesMut) -> Result<Self, Error> {
        let pid = Pid::from_buffer(buf)?;
        let mut topics: Vec<SubscribeTopic> = Vec::new();
        while buf.len() != 0 {
            let topic_path = read_string(buf)?;
            let qos = QoS::from_u8(buf.split_to(1).get_u8())?;
            let topic = SubscribeTopic { topic_path, qos };
            topics.push(topic);
        }
        Ok(Subscribe { pid, topics })
    }

    pub(crate) fn to_buffer(&self, buf: &mut BytesMut) -> Result<(), Error> {
        let header: u8 = 0b10000010;
        check_remaining(buf, 1)?;
        buf.put_u8(header);

        // Length: pid(2) + topic.for_each(2+len + qos(1))
        let mut length = 2;
        for topic in &self.topics {
            length += topic.topic_path.len() + 2 + 1;
        }
        write_length(length, buf)?;

        // Pid
        self.pid.to_buffer(buf)?;

        // Topics
        for topic in &self.topics {
            write_string(topic.topic_path.as_ref(), buf)?;
            buf.put_u8(topic.qos.to_u8());
        }

        Ok(())
    }
}

impl Unsubscribe {
    pub(crate) fn from_buffer(buf: &mut BytesMut) -> Result<Self, Error> {
        let pid = Pid::from_buffer(buf)?;
        let mut topics: Vec<String> = Vec::new();
        while buf.len() != 0 {
            let topic_path = read_string(buf)?;
            topics.push(topic_path);
        }
        Ok(Unsubscribe { pid, topics })
    }

    pub(crate) fn to_buffer(&self, buf: &mut BytesMut) -> Result<(), Error> {
        let header: u8 = 0b10100010;
        let mut length = 2;
        for topic in &self.topics {
            length += 2 + topic.len();
        }
        check_remaining(buf, 1)?;
        buf.put_u8(header);

        write_length(length, buf)?;
        self.pid.to_buffer(buf)?;
        for topic in &self.topics {
            write_string(topic.as_ref(), buf)?;
        }
        Ok(())
    }
}

impl Suback {
    pub(crate) fn from_buffer(buf: &mut BytesMut) -> Result<Self, Error> {
        let pid = Pid::from_buffer(buf)?;
        let mut return_codes: Vec<SubscribeReturnCodes> = Vec::new();
        while buf.len() != 0 {
            let code = buf.split_to(1).get_u8();
            let r = if code == 0x80 {
                SubscribeReturnCodes::Failure
            } else {
                SubscribeReturnCodes::Success(QoS::from_u8(code)?)
            };
            return_codes.push(r);
        }
        Ok(Suback { return_codes, pid })
    }
    pub(crate) fn to_buffer(&self, buf: &mut BytesMut) -> Result<(), Error> {
        let header: u8 = 0b10010000;
        let length = 2 + self.return_codes.len();
        check_remaining(buf, 1)?;
        buf.put_u8(header);

        write_length(length, buf)?;
        self.pid.to_buffer(buf)?;
        for rc in &self.return_codes {
            buf.put_u8(rc.to_u8());
        }
        Ok(())
    }
}
