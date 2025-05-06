use mqttrs::QosPid;
use mqttrs_tokio_client_example::mqtt::{MqttListener, MqttOptions};

#[tokio::main]
async fn main() {

    let config = MqttOptions { 
        client: "mqttrs-client".into(), 
        host: "localhost".into(), 
        port: 1883, 
        username: Some("username".into()), 
        password: Some("password".into()), 
        keep_alive: 60
    };

    let client = MqttListener::init(config);

    for i in 0..10 {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        client.publish(
            "test/topic", 
            QosPid::AtMostOnce,
            true,
            format!("Say hi {} times!", i));
    }
}