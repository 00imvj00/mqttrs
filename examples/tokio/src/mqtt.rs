use mqttrs::{
    Connect, ConnectReturnCode, Packet, Pid, Protocol, Publish, QoS, QosPid, Subscribe,
    SubscribeTopic, decode_slice, encode_slice,
};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, sync::mpsc};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct MqttOptions {
    pub client: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub keep_alive: u16
}

#[derive(Debug, Clone)]
pub struct MqttListener {
    config: MqttOptions,
    sender: mpsc::Sender<MqttCommand>,
}

#[derive(Debug)]
enum MqttCommand {
    Publish {
        topic: String,
        qos: QosPid,
        retain: bool,
        payload: Vec<u8>,
    }
}

impl MqttListener {
    pub fn init(config: MqttOptions) -> Self {
        let (tx, rx) = mpsc::channel::<MqttCommand>(100);
        let mqtt = Self {
            config: config,
            sender: tx.clone(),
        };
        mqtt.spawn_thread(rx);
        mqtt
    }

    pub fn publish(&self, topic: impl Into<String>, qos: QosPid, retain: bool, payload: impl Into<Vec<u8>>) {
        let topic = topic.into();
        let payload = payload.into();
        if let Err(e) = self.sender.try_send(MqttCommand::Publish {
            topic: topic.to_string(),
            payload: payload.to_vec(),
            qos,
            retain,
        }) {
            println!("unable to send publish command to thread: {}", e);
        };
    }

    fn spawn_thread(&self, mut rx: mpsc::Receiver<MqttCommand>) {
        let config = self.config.clone();
        tokio::spawn(async move {
            let mut backoff = 1;

            'reconnect: loop {
                println!("Connecting to MQTT broker...");
                let addr = format!("{}:{}", config.host, config.port);

                let stream = match TcpStream::connect(&addr).await {
                    Ok(s) => {
                        println!("Connected to broker");
                        backoff = 1; // reset on success
                        s
                    }
                    Err(e) => {
                        println!("Connect error: {}", e);
                        let delay = Duration::from_secs((backoff * 2).min(60));
                        println!("Retrying in {} seconds", delay.as_secs());
                        tokio::time::sleep(delay).await;
                        backoff *= 2;
                        continue 'reconnect;
                    }
                };

                let (mut mqtt_read, mut mqtt_write) = stream.into_split();

                // CONNECT
                let connect_pkt = Packet::Connect(Connect {
                    protocol: Protocol::MQTT311,
                    keep_alive: config.keep_alive,
                    client_id: &config.client,
                    clean_session: true,
                    last_will: None,
                    username: config.username.as_deref(),
                    password: config.password.as_deref().map(|s| s.as_bytes()),
                });

                let mut buf = [0u8; 1024];
                let len = encode_slice(&connect_pkt, &mut buf).expect("Invalid CONNECT");
                if mqtt_write.write_all(&buf[..len]).await.is_err() {
                    continue 'reconnect;
                }

                let mut res = [0u8; 1024];
                let n = mqtt_read.read(&mut res).await.unwrap_or(0);
                if n == 0 {
                    continue 'reconnect;
                }

                match decode_slice(&res[..n]) {
                    Ok(Some(Packet::Connack(connack)))
                        if connack.code == ConnectReturnCode::Accepted =>
                    {
                        println!("Connection accepted");
                    }
                    _ => {
                        println!("Connection rejected or bad CONNACK");
                        continue 'reconnect;
                    }
                }

                // SUBSCRIBE
                let pid = Pid::new();
                let subscribe_pkt = Packet::Subscribe(Subscribe {
                    pid,
                    topics: vec![SubscribeTopic {
                        topic_path: "test/topic".into(),
                        qos: QoS::AtMostOnce,
                    }],
                });

                let len = encode_slice(&subscribe_pkt, &mut buf).expect("Invalid SUBSCRIBE");
                if mqtt_write.write_all(&buf[..len]).await.is_err() {
                    continue 'reconnect;
                }

                let n = mqtt_read.read(&mut res).await.unwrap_or(0);
                if n == 0 {
                    continue 'reconnect;
                }

                match decode_slice(&res[..n]) {
                    Ok(Some(Packet::Suback(ack))) if ack.pid == pid => {
                        println!("Subscribed");
                    }
                    _ => {
                        println!("SUBACK failed");
                        continue 'reconnect;
                    }
                }

                let ping_interval = Duration::from_secs(config.keep_alive as u64 / 2);

                let mut ping_buf = [0u8; 1024];
                let mut recv_buf = [0u8; 1024];
                let mut publish_buf = [0u8; 1024];

                let mut last_ping = tokio::time::Instant::now();

                loop {
                    tokio::select! {
                        biased;

                        Some(cmd) = rx.recv() => {
                            match cmd {
                                MqttCommand::Publish { topic, payload, qos, retain } => {
                                    let publish_pkt = Packet::Publish(Publish {
                                        dup: false,
                                        qospid: qos,
                                        retain: retain,
                                        topic_name: &topic,
                                        payload: &payload,
                                    });

                                    if let Ok(len) = encode_slice(&publish_pkt, &mut publish_buf) {
                                        if mqtt_write.write_all(&publish_buf[..len]).await.is_err() {
                                            println!("Publish failed");
                                            break;
                                        } else {
                                            println!("Published to topic");
                                        }
                                    }
                                }
                            }
                        }

                        _ = tokio::time::sleep_until(last_ping + ping_interval) => {
                            let len = encode_slice(&Packet::Pingreq, &mut ping_buf).expect("Invalid PINGREQ");
                            if mqtt_write.write_all(&ping_buf[..len]).await.is_err() {
                                println!("PINGREQ failed");
                                break;
                            }
                            println!("Sent PINGREQ");
                            last_ping = tokio::time::Instant::now();
                        }

                        n = mqtt_read.read(&mut recv_buf) => {
                            let n = match n {
                                Ok(n) if n > 0 => n,
                                _ => break,
                            };

                            match decode_slice(&recv_buf[..n]) {
                                Ok(Some(pkt)) => match pkt {
                                    Packet::Pingresp => {
                                        println!("Received PINGRESP");
                                    }
                                    Packet::Publish(p) => {
                                        println!("Received PUBLISH: topic = {}", p.topic_name);
                                    }
                                    _ => {}
                                },
                                _ => {
                                    println!("Failed to decode incoming packet");
                                }
                            }
                        }
                    }
                }

                println!("Disconnected from broker");
                let delay = Duration::from_secs((backoff * 2).min(60));
                println!("Reconnecting in {} seconds...", delay.as_secs());
                tokio::time::sleep(delay).await;
                backoff *= 2;
            }
        });
    }
}