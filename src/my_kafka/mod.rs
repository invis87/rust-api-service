extern crate kafka;

use std::fmt::{ Write, Debug };
use std::time::Duration;
use kafka::producer::{Producer, Record, RequiredAcks};
use std::fmt::{Formatter, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use std::u128;

pub struct KafkaWriter {
    producer: Producer,
}

impl KafkaWriter {
    pub fn new(hosts: Vec<String>) -> Self {
       let producer = Producer::from_hosts(hosts)
        .with_ack_timeout(Duration::from_secs(1))
        .with_required_acks(RequiredAcks::One)
        .create()
        .unwrap();
        KafkaWriter { producer }
    }

    pub fn send_string(&mut self, topic: &str, msg: &str, log: &slog::Logger) {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");

        let x: [u8; 16] = since_the_epoch.as_millis().to_ne_bytes();
        let message_key: &[u8] = x.as_ref();

        let kafka_record = Record::from_key_value(topic, message_key, msg.as_bytes()).with_partition(1);
        let send_result = self.producer.send(&kafka_record);
        match send_result {
            Ok(result) => info!(log, "message '{}' successfully sends to topic '{}'", msg, topic),
            Err(error) => error!(log, "fail to send message to kafka, reason: {:?}", error)
        }
    }
}