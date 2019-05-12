#[macro_use]
extern crate slog;
extern crate actix_web;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate failure;
extern crate reqwest;

mod data;
mod logging;
mod handlers;
mod my_kafka;

use actix_web::{http, server, App};
use std::sync::{Arc, Mutex};
use my_kafka::KafkaWriter;
use crate::my_kafka::KafkaReader;

pub struct AppState {
    log: slog::Logger,
    kafka_writer: Arc<Mutex<KafkaWriter>>,
    kafka_reader: Arc<Mutex<KafkaReader>>,
}

fn main() {
    let kafka_topic = "rust-api-data".to_owned();
    let kafka_brokers = vec!["localhost:9092".to_owned(), "localhost:9093".to_owned(), "localhost:9094".to_owned()];

    let kafka_writer = KafkaWriter::new(kafka_brokers.clone());
    let arc_kafka_writer = Arc::new(Mutex::new(kafka_writer));

    let kafka_reader = KafkaReader::new(kafka_topic.clone(), kafka_brokers);
    let arc_kafka_reader = Arc::new(Mutex::new(kafka_reader));

    let log = logging::setup_logging();
    info!(log, "Starting server on localhost:8080");

    server::new(move || App::with_state(AppState {
        log: log.clone(),
        kafka_writer: Arc::clone(&arc_kafka_writer),
        kafka_reader: Arc::clone(&arc_kafka_reader),
    })
        .resource("/calculate", |r| {
            r.method(http::Method::POST)
                .with_config(handlers::calculate, |cfg| {(cfg.0).1.error_handler(handlers::json_error_handler);})
        })
        .resource("/health", |r| { r.method(http::Method::GET).with(handlers::health) })
        .resource("/produce", |r| {
        r.method(http::Method::POST)
            .with_config(handlers::produce, |cfg| {(cfg.0).1.error_handler(handlers::json_error_handler);})
    })
        .resource("/consume", |r| { r.method(http::Method::GET).with(handlers::consume) })
        .finish())
        .bind("0.0.0.0:8080")
        .unwrap()
        .run();
}