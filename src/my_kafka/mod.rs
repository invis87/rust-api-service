extern crate kafka;

use std::fmt::Write;
use std::time::Duration;
use kafka::producer::{Producer, Record, RequiredAcks};
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
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

pub struct KafkaReader {
    consumer: Consumer,
}

impl KafkaReader {
    pub fn new(topic: String, hosts: Vec<String>) -> Self {
        let group_name = format!("{}-group", topic);
        let consumer = Consumer::from_hosts(hosts)
            .with_topic(topic)
            .with_fallback_offset(FetchOffset::Earliest)
            .with_group(group_name)
            .with_offset_storage(GroupOffsetStorage::Kafka)
            .create()
            .expect("fail to create kafka consumer");
        KafkaReader { consumer }
    }

    pub fn read_next(&mut self, log: &slog::Logger) -> String {
        let mut buf = String::with_capacity(100);
        let polled = self.consumer.poll().expect("fail to get message from kafka");

        for ms in polled.iter() {
            for m in ms.messages() {
                info!(log, "consume kafka message, offset={}", m.offset);
                let message_key: &[u8] = m.key;
                let message_value: &[u8] = m.value;
                let key = String::from_utf8(message_key.to_vec()).unwrap();
                let value = String::from_utf8(message_value.to_vec()).unwrap();
                write!(&mut buf, "key={}, value={}", key, value).expect("fail to write to string buffer");
            }
            self.consumer.consume_messageset(ms).expect("fail to mark kafka messages sa consumed");
        }
        self.consumer.commit_consumed().expect("fail to commit messages as consumer");
        buf
    }
}