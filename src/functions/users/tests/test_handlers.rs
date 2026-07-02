mod common;

use common::*;
use users_lambda::errors::AppError;
use users_lambda::handlers::{
    create_user, decode_pagination_token, delete_user, encode_pagination_token, extract_userid,
    get_user, list_users, options_response, update_user,
};

// --- options_response ---

#[test]
fn test_options_response_cors_headers() {
    let resp = options_response();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers()["access-control-allow-origin"], "*");
    assert_eq!(resp.headers()["access-control-max-age"], "3600");
    let methods = resp.headers()["access-control-allow-methods"]
        .to_str()
        .unwrap();
    assert!(methods.contains("GET"));
    assert!(methods.contains("PUT"));
    assert!(methods.contains("DELETE"));
    assert!(methods.contains("OPTIONS"));
}

// --- pagination token ---

#[test]
fn test_encode_decode_roundtrip() {
    let original = "user-id-12345";
    let encoded = encode_pagination_token(original);
    let decoded = decode_pagination_token(&encoded).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_decode_invalid_base64() {
    let result = decode_pagination_token("not!valid!base64!!!");
    assert!(result.is_err());
}

#[test]
fn test_decode_invalid_utf8() {
    use base64::Engine;
    let invalid_bytes: Vec<u8> = vec![0xFF, 0xFE, 0xFD];
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&invalid_bytes);
    let result = decode_pagination_token(&encoded);
    assert!(result.is_err());
}

#[test]
fn test_encode_empty_string() {
    let encoded = encode_pagination_token("");
    let decoded = decode_pagination_token(&encoded).unwrap();
    assert_eq!(decoded, "");
}

// --- extract_userid ---

#[test]
fn test_extract_userid_present() {
    let req = request_with_userid("GET", "abc-123", None);
    let result = extract_userid(&req);
    assert_eq!(result.unwrap(), "abc-123");
}

#[test]
fn test_extract_userid_missing() {
    let req = request_no_userid("GET", None);
    let result = extract_userid(&req);
    assert!(result.is_err());
}

// --- get_user ---

#[tokio::test]
async fn test_get_user_found() {
    let body = r#"{"Item":{"userid":{"S":"u1"},"email":{"S":"a@b.com"},"name":{"S":"Alice"},"phone":{"S":"+123"}}}"#;
    let state = test_state(vec![dynamo_ok(body)]);
    let req = request_with_userid("GET", "u1", None);

    let resp = get_user(&state, &req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = response_body_string(&resp);
    assert!(body.contains("Alice"));
    assert!(body.contains("a@b.com"));
}

#[tokio::test]
async fn test_get_user_not_found() {
    let state = test_state(vec![dynamo_ok("{}")]);
    let req = request_with_userid("GET", "nonexistent", None);

    let result = get_user(&state, &req).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn test_get_user_dynamo_error() {
    let state = test_state(vec![dynamo_error(500, INTERNAL_SERVER_ERROR)]);
    let req = request_with_userid("GET", "u1", None);

    let result = get_user(&state, &req).await;
    assert!(matches!(result, Err(AppError::Internal(_))));
}

// --- create_user ---

#[tokio::test]
async fn test_create_user_success() {
    let state = test_state(vec![dynamo_ok("{}")]);
    let body = r#"{"email":"new@example.com","name":"New User"}"#;
    let req = request_no_userid("PUT", Some(body));

    let resp = create_user(&state, &req).await.unwrap();
    assert_eq!(resp.status(), 201);
    let resp_body = response_body_string(&resp);
    assert!(resp_body.contains("new@example.com"));
    assert!(resp_body.contains("userid"));
}

#[tokio::test]
async fn test_create_user_empty_body() {
    let state = test_state(vec![]);
    let req = request_no_userid("PUT", None);

    let result = create_user(&state, &req).await;
    assert!(matches!(result, Err(AppError::ValidationError(msg)) if msg.contains("body is required")));
}

#[tokio::test]
async fn test_create_user_invalid_json() {
    let state = test_state(vec![]);
    let req = request_no_userid("PUT", Some("not json"));

    let result = create_user(&state, &req).await;
    assert!(matches!(result, Err(AppError::ValidationError(msg)) if msg.contains("invalid JSON")));
}

#[tokio::test]
async fn test_create_user_validation_fails() {
    let state = test_state(vec![]);
    let body = r#"{"email":"","name":"Test"}"#;
    let req = request_no_userid("PUT", Some(body));

    let result = create_user(&state, &req).await;
    assert!(matches!(result, Err(AppError::ValidationError(_))));
}

#[tokio::test]
async fn test_create_user_conflict() {
    let state = test_state(vec![dynamo_conditional_check_failed()]);
    let body = r#"{"email":"a@b.com","name":"Test"}"#;
    let req = request_no_userid("PUT", Some(body));

    let result = create_user(&state, &req).await;
    assert!(matches!(result, Err(AppError::Conflict(_))));
}

// --- update_user ---

#[tokio::test]
async fn test_update_user_success() {
    let dynamo_body =
        r#"{"Attributes":{"userid":{"S":"u1"},"email":{"S":"updated@b.com"},"name":{"S":"Alice"}}}"#;
    let state = test_state(vec![dynamo_ok(dynamo_body)]);
    let body = r#"{"email":"updated@b.com"}"#;
    let req = request_with_userid("PUT", "u1", Some(body));

    let resp = update_user(&state, &req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let resp_body = response_body_string(&resp);
    assert!(resp_body.contains("updated@b.com"));
}

#[tokio::test]
async fn test_update_user_not_found() {
    let state = test_state(vec![dynamo_conditional_check_failed()]);
    let body = r#"{"name":"New Name"}"#;
    let req = request_with_userid("PUT", "nonexistent", Some(body));

    let result = update_user(&state, &req).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn test_update_user_empty_body() {
    let state = test_state(vec![]);
    let req = request_with_userid("PUT", "u1", None);

    let result = update_user(&state, &req).await;
    assert!(matches!(result, Err(AppError::ValidationError(msg)) if msg.contains("body is required")));
}

#[tokio::test]
async fn test_update_user_invalid_json() {
    let state = test_state(vec![]);
    let req = request_with_userid("PUT", "u1", Some("{bad"));

    let result = update_user(&state, &req).await;
    assert!(matches!(result, Err(AppError::ValidationError(msg)) if msg.contains("invalid JSON")));
}

#[tokio::test]
async fn test_update_user_all_fields() {
    let dynamo_body = r#"{"Attributes":{"userid":{"S":"u1"},"email":{"S":"new@b.com"},"name":{"S":"New"},"phone":{"S":"+999"}}}"#;
    let state = test_state(vec![dynamo_ok(dynamo_body)]);
    let body = r#"{"email":"new@b.com","name":"New","phone":"+999"}"#;
    let req = request_with_userid("PUT", "u1", Some(body));

    let resp = update_user(&state, &req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

// --- delete_user ---

#[tokio::test]
async fn test_delete_user_success() {
    let state = test_state(vec![dynamo_ok("{}")]);
    let req = request_with_userid("DELETE", "u1", None);

    let resp = delete_user(&state, &req).await.unwrap();
    assert_eq!(resp.status(), 204);
}

#[tokio::test]
async fn test_delete_user_not_found() {
    let state = test_state(vec![dynamo_conditional_check_failed()]);
    let req = request_with_userid("DELETE", "nonexistent", None);

    let result = delete_user(&state, &req).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn test_delete_user_dynamo_error() {
    let state = test_state(vec![dynamo_error(500, INTERNAL_SERVER_ERROR)]);
    let req = request_with_userid("DELETE", "u1", None);

    let result = delete_user(&state, &req).await;
    assert!(matches!(result, Err(AppError::Internal(_))));
}

// --- list_users ---

#[tokio::test]
async fn test_list_users_default_page() {
    let dynamo_body = r#"{"Items":[{"userid":{"S":"u1"},"email":{"S":"a@b.com"},"name":{"S":"Alice"}}],"Count":1,"ScannedCount":1}"#;
    let state = test_state(vec![dynamo_ok(dynamo_body)]);
    let req = request_no_userid("GET", None);

    let resp = list_users(&state, &req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = response_body_string(&resp);
    assert!(body.contains("Alice"));
}

#[tokio::test]
async fn test_list_users_empty_table() {
    let dynamo_body = r#"{"Items":[],"Count":0,"ScannedCount":0}"#;
    let state = test_state(vec![dynamo_ok(dynamo_body)]);
    let req = request_no_userid("GET", None);

    let resp = list_users(&state, &req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = response_body_string(&resp);
    assert!(body.contains(r#""users":[]"#));
}

#[tokio::test]
async fn test_list_users_with_pagination() {
    let dynamo_body = r#"{"Items":[{"userid":{"S":"u1"},"email":{"S":"a@b.com"},"name":{"S":"Alice"}}],"Count":1,"ScannedCount":1,"LastEvaluatedKey":{"userid":{"S":"u1"}}}"#;
    let state = test_state(vec![dynamo_ok(dynamo_body)]);
    let req = request_no_userid("GET", None);

    let resp = list_users(&state, &req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = response_body_string(&resp);
    assert!(body.contains("next_token"));
}

#[tokio::test]
async fn test_list_users_dynamo_error() {
    let state = test_state(vec![dynamo_error(500, INTERNAL_SERVER_ERROR)]);
    let req = request_no_userid("GET", None);

    let result = list_users(&state, &req).await;
    assert!(matches!(result, Err(AppError::Internal(_))));
}
