extern crate kafka;

use std::time::Duration;
use kafka::producer::{Producer, Record, RequiredAcks};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct KafkaWriter {
    producer: Producer,
}

impl KafkaWriter {
    pub fn new(hosts: Vec<String>) -> Self {
       let producer = Producer::from_hosts(hosts)
        .with_ack_timeout(Duration::from_secs(1))
        .with_required_acks(RequiredAcks::One)
        .create()
        .expect("fail to create kafka producer");
        KafkaWriter { producer }
    }

    pub fn send_string(&mut self, topic: &str, msg: &str, log: &slog::Logger) {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let message_key = since_the_epoch.as_millis().to_string();

        let kafka_record = Record::from_key_value(topic, message_key.as_bytes(), msg.as_bytes());
        let send_result = self.producer.send(&kafka_record);
        match send_result {
            Ok(_) => info!(log, "message '{}' successfully sends to topic '{}'", msg, topic),
            Err(error) => error!(log, "fail to send message to kafka, reason: {:?}", error)
        }
    }
}