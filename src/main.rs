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

use actix_web::{http, server, App};
use actix_web::Responder;

mod data;
mod logging;
mod handlers;


#[derive(Debug)]
pub struct AppState {
    log: slog::Logger,
}

fn main() {
    let log = logging::setup_logging();
    info!(log, "Server Started on localhost:8080");
    server::new(move || App::with_state(AppState{log: log.clone()})
        .resource("/calculate", |r| {
            r.method(http::Method::POST)
                .with_config(handlers::calculate, |cfg| {(cfg.0).1.error_handler(handlers::json_error_handler);})
        })
        .resource("/health", |r| { r.method(http::Method::GET).f(handlers::health) })
        .finish())
        .bind("0.0.0.0:8080")
        .unwrap()
        .run();
}