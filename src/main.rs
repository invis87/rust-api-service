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

pub struct AppState {
    log: slog::Logger,
    kafka_writer: Arc<Mutex<KafkaWriter>>,
}

fn main() {
    let kafka_brokers = vec!["localhost:9092".to_owned(), "localhost:9093".to_owned(), "localhost:9094".to_owned()];
    let kafka_writer = KafkaWriter::new(kafka_brokers);
    let arc_kafka_writer = Arc::new(Mutex::new(kafka_writer));

    let log = logging::setup_logging();
    info!(log, "Starting server on localhost:8080");

    server::new(move || App::with_state(AppState {
        log: log.clone(),
        kafka_writer: Arc::clone(&arc_kafka_writer),
    })
        .resource("/calculate", |r| {
            r.method(http::Method::POST)
                .with_config(handlers::calculate, |cfg| {(cfg.0).1.error_handler(handlers::json_error_handler);})
        })
        .resource("/health", |r| { r.method(http::Method::GET).with(handlers::health) })
        .finish())
        .bind("0.0.0.0:8080")
        .unwrap()
        .run();
}