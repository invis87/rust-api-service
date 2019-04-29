use actix_web::{error, Error, HttpRequest, HttpResponse, Json, Path, Responder};
use super::data::{CalculateRequest, CalculateResponse};
use super::AppState;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Fail, Debug)]
pub enum CalculatorError {
    #[fail(display = "External Service Error")]
    ExternalServiceError,
}

impl error::ResponseError for CalculatorError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            CalculatorError::ExternalServiceError => HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("external service error"),
        }
    }
}

pub fn calculate(
    (req, calc_req): (HttpRequest<AppState>, Json<CalculateRequest>),
) -> Result<Json<CalculateResponse>, CalculatorError> {
    let log = &req.state().log;
    info!(log, "calculating {:?}", calc_req);
    thread::sleep(Duration::from_secs(2));
    let millis = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis();
    Result::Ok(CalculateResponse{result: millis.to_string()}).map(Json)
}

pub fn health(req: &HttpRequest<AppState>) -> impl Responder {
    let log = &req.state().log;
    info!(log, "HEALTH requested");
    "OK".to_string()
}


pub fn json_error_handler(err: error::JsonPayloadError, _: &HttpRequest<AppState>) -> Error {
    error::InternalError::from_response(
        "",
        HttpResponse::BadRequest()
            .content_type("application/json")
            .body(format!(r#"{{"error": "{}"}}"#, err)),
    )
        .into()
}