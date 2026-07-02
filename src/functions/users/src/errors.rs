use lambda_http::{Body, Response};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    ValidationError(String),
    Conflict(String),
    MethodNotAllowed,
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {msg}"),
            AppError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            AppError::Conflict(msg) => write!(f, "Conflict: {msg}"),
            AppError::MethodNotAllowed => write!(f, "Method not allowed"),
            AppError::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl AppError {
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::NotFound(_) => 404,
            AppError::ValidationError(_) => 400,
            AppError::Conflict(_) => 409,
            AppError::MethodNotAllowed => 405,
            AppError::Internal(_) => 500,
        }
    }

    pub fn into_response(self) -> Response<Body> {
        let (status_code, message) = match &self {
            AppError::NotFound(msg) => (404, msg.clone()),
            AppError::ValidationError(msg) => (400, msg.clone()),
            AppError::Conflict(msg) => (409, msg.clone()),
            AppError::MethodNotAllowed => (405, "Method not allowed".to_string()),
            AppError::Internal(_) => (500, "Internal server error".to_string()),
        };

        let body = json!({ "message": message }).to_string();

        Response::builder()
            .status(status_code)
            .header("content-type", "application/json")
            .header("cache-control", "no-store, no-cache, must-revalidate")
            .header("x-content-type-options", "nosniff")
            .body(Body::Text(body))
            .unwrap()
    }
}

pub fn success_response<T: serde::Serialize>(status_code: u16, body: &T) -> Response<Body> {
    let body_str = serde_json::to_string(body).unwrap_or_else(|_| "{}".to_string());

    Response::builder()
        .status(status_code)
        .header("content-type", "application/json")
        .header("cache-control", "no-store, no-cache, must-revalidate")
        .header("x-content-type-options", "nosniff")
        .body(Body::Text(body_str))
        .unwrap()
}
