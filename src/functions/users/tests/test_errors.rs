use lambda_http::Body;
use serde::Serialize;
use users_lambda::errors::{success_response, AppError};

fn response_body_string(resp: &lambda_http::Response<Body>) -> String {
    match resp.body() {
        Body::Text(t) => t.clone(),
        _ => panic!("expected text body"),
    }
}

#[test]
fn test_not_found_response() {
    let err = AppError::NotFound("User xyz not found".into());
    let resp = err.into_response();
    assert_eq!(resp.status(), 404);
    let body = response_body_string(&resp);
    assert!(body.contains("User xyz not found"));
    assert_eq!(resp.headers()["content-type"], "application/json");
    assert_eq!(resp.headers()["x-content-type-options"], "nosniff");
}

#[test]
fn test_validation_error_response() {
    let err = AppError::ValidationError("email is required".into());
    let resp = err.into_response();
    assert_eq!(resp.status(), 400);
    let body = response_body_string(&resp);
    assert!(body.contains("email is required"));
}

#[test]
fn test_conflict_response() {
    let err = AppError::Conflict("user already exists".into());
    let resp = err.into_response();
    assert_eq!(resp.status(), 409);
    let body = response_body_string(&resp);
    assert!(body.contains("user already exists"));
}

#[test]
fn test_method_not_allowed_response() {
    let err = AppError::MethodNotAllowed;
    let resp = err.into_response();
    assert_eq!(resp.status(), 405);
    let body = response_body_string(&resp);
    assert!(body.contains("Method not allowed"));
}

#[test]
fn test_internal_response_does_not_leak() {
    let err = AppError::Internal("secret database connection string".into());
    let resp = err.into_response();
    assert_eq!(resp.status(), 500);
    let body = response_body_string(&resp);
    assert!(body.contains("Internal server error"));
    assert!(!body.contains("secret"));
}

#[test]
fn test_success_response_200() {
    #[derive(Serialize)]
    struct Msg {
        hello: String,
    }
    let msg = Msg {
        hello: "world".into(),
    };
    let resp = success_response(200, &msg);
    assert_eq!(resp.status(), 200);
    let body = response_body_string(&resp);
    assert!(body.contains("world"));
    assert_eq!(resp.headers()["content-type"], "application/json");
    assert_eq!(
        resp.headers()["cache-control"],
        "no-store, no-cache, must-revalidate"
    );
}

#[test]
fn test_success_response_201() {
    #[derive(Serialize)]
    struct Id {
        id: u32,
    }
    let resp = success_response(201, &Id { id: 42 });
    assert_eq!(resp.status(), 201);
}

#[test]
fn test_display_all_variants() {
    assert!(format!("{}", AppError::NotFound("x".into())).contains("Not found: x"));
    assert!(
        format!("{}", AppError::ValidationError("y".into())).contains("Validation error: y")
    );
    assert!(format!("{}", AppError::Conflict("z".into())).contains("Conflict: z"));
    assert!(format!("{}", AppError::MethodNotAllowed).contains("Method not allowed"));
    assert!(format!("{}", AppError::Internal("i".into())).contains("Internal error: i"));
}
