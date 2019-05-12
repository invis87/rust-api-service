use super::data::{ProduceRequest, ProduceResponse};
use super::AppState;

use actix_web::{Error, HttpRequest, HttpResponse, Json};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::prelude::*;
use actix_web::AsyncResponder;
use actix_web::FutureResponse;
use crate::my_kafka::KafkaWriter;
use std::sync::Mutex;
use std::sync::Arc;
use crate::my_kafka::KafkaReader;

#[derive(Fail, Debug)]
pub enum ServiceError {
    #[fail(display = "External Service Error")]
    ExternalServiceError,

    #[fail(display = "External Service Error")]
    InformativeError(String),
}

impl actix_web::error::ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::ExternalServiceError => HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("external service error"),

            ServiceError::InformativeError(message) => HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(message),
        }
    }
}

pub fn do_async(req: HttpRequest<AppState>) -> FutureResponse<String, ServiceError>  {
    let log = &req.state().log;
    info!(log, "calculating ...");

    let future_result = future::lazy(|| -> Result<String, ServiceError> {
        thread::sleep(Duration::from_secs(2));
        SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|_| ServiceError::ExternalServiceError)
            .map(|dur| dur.as_millis().to_string())

    });

    future_result.responder()
}

pub fn health(req: HttpRequest<AppState>) -> FutureResponse<String> {
    let log: &slog::Logger = &req.state().log;
    info!(log, "Health requested");

    future::ok("OK".to_string()).responder()
}

pub fn produce(
    (req, produce_req): (HttpRequest<AppState>, Json<ProduceRequest>),
) -> Result<Json<ProduceResponse>, ServiceError> {
    let log: &slog::Logger = &req.state().log;
    info!(log, "Produce requested: {}", produce_req);

    let topic = &produce_req.topic;
    let message = &produce_req.message;
    let kafka_writer_arc: &Arc<Mutex<KafkaWriter>> = &req.state().kafka_writer;
    let mut kafka_writer = kafka_writer_arc.lock().expect("fail to get kafka writer");

    let produce_result = kafka_writer.send_string(topic, message, log);
    match produce_result {
        Ok(_) => Result::Ok(ProduceResponse{result: format!("message send to topic '{}'", topic)}).map(Json),
        Err(error) => Result::Err(ServiceError::InformativeError(error.to_string()))
    }
}

pub fn consume(req: HttpRequest<AppState>) -> FutureResponse<String> {
    let log: &slog::Logger = &req.state().log;
    info!(log, "consume requested");

    let kafka_reader_arc: &Arc<Mutex<KafkaReader>> = &req.state().kafka_reader;
    let mut kafka_reader = kafka_reader_arc.lock().expect("fail to get kafka reader");

    let messages_from_kafka = kafka_reader.read_next(log);

    future::ok(messages_from_kafka).responder()
}

pub fn json_error_handler(err: actix_web::error::JsonPayloadError, _: &HttpRequest<AppState>) -> Error {
    actix_web::error::InternalError::from_response(
        "",
        HttpResponse::BadRequest()
            .content_type("application/json")
            .body(format!(r#"{{"error": "{}"}}"#, err)),
    )
        .into()
}