mod common;

use common::*;
use lambda_http::Body;
use users_lambda::handle_request;

#[tokio::test]
async fn test_route_options() {
    let state = test_state(vec![]);
    let req = request_no_userid("OPTIONS", None);
    let resp = handle_request(req, &state).await.unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.headers().contains_key("access-control-allow-origin"));
}

#[tokio::test]
async fn test_route_unsupported_method() {
    let state = test_state(vec![]);
    let req = request_no_userid("POST", None);
    let resp = handle_request(req, &state).await.unwrap();
    assert_eq!(resp.status(), 405);
}

#[tokio::test]
async fn test_route_get_with_userid() {
    let body = r#"{"Item":{"userid":{"S":"u1"},"email":{"S":"a@b.com"},"name":{"S":"Alice"}}}"#;
    let state = test_state(vec![dynamo_ok(body)]);
    let req = request_with_userid("GET", "u1", None);
    let resp = handle_request(req, &state).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_route_get_without_userid() {
    let body = r#"{"Items":[],"Count":0,"ScannedCount":0}"#;
    let state = test_state(vec![dynamo_ok(body)]);
    let req = request_no_userid("GET", None);
    let resp = handle_request(req, &state).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_route_put_without_userid_creates() {
    let state = test_state(vec![dynamo_ok("{}")]);
    let req: lambda_http::Request = http::Request::builder()
        .method("PUT")
        .uri("/users")
        .body(Body::Text(r#"{"email":"a@b.com","name":"Test"}"#.into()))
        .unwrap()
        .into();
    let resp = handle_request(req, &state).await.unwrap();
    assert_eq!(resp.status(), 201);
}

#[tokio::test]
async fn test_route_delete_with_userid() {
    let state = test_state(vec![dynamo_ok("{}")]);
    let req = request_with_userid("DELETE", "u1", None);
    let resp = handle_request(req, &state).await.unwrap();
    assert_eq!(resp.status(), 204);
}

#[tokio::test]
async fn test_route_error_returns_response_not_error() {
    let state = test_state(vec![dynamo_ok("{}")]);
    let req = request_with_userid("GET", "nope", None);
    let resp = handle_request(req, &state).await.unwrap();
    assert_eq!(resp.status(), 404);
}
