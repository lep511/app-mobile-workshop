use aws_credential_types::Credentials;
use aws_sdk_dynamodb::config::Region;
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_smithy_runtime::client::http::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_types::body::SdkBody;
use lambda_http::aws_lambda_events::query_map::QueryMap;
use lambda_http::{Body, Request, RequestExt};
use std::collections::HashMap;
use users_lambda::AppState;

pub fn mock_dynamo_client(events: Vec<ReplayEvent>) -> DynamoClient {
    let http_client = StaticReplayClient::new(events);
    let config = aws_sdk_dynamodb::Config::builder()
        .behavior_version(aws_sdk_dynamodb::config::BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .http_client(http_client)
        .credentials_provider(Credentials::for_tests())
        .build();
    DynamoClient::from_conf(config)
}

pub fn test_state(events: Vec<ReplayEvent>) -> AppState {
    AppState::new(mock_dynamo_client(events), "test-table".to_string())
}

pub fn dynamo_ok(body: &str) -> ReplayEvent {
    dynamo_response(200, body)
}

#[allow(dead_code)]
pub fn dynamo_error(status: u16, body: &str) -> ReplayEvent {
    dynamo_response(status, body)
}

fn dynamo_response(status: u16, body: &str) -> ReplayEvent {
    ReplayEvent::new(
        http::Request::builder()
            .body(SdkBody::from(""))
            .unwrap(),
        http::Response::builder()
            .status(status)
            .body(SdkBody::from(body))
            .unwrap(),
    )
}

pub fn request_with_userid(method: &str, userid: &str, body: Option<&str>) -> Request {
    let b = match body {
        Some(t) => Body::Text(t.to_string()),
        None => Body::Empty,
    };
    let req = http::Request::builder()
        .method(method)
        .uri(format!("/users/{}", userid))
        .body(b)
        .unwrap();
    let mut params = HashMap::new();
    params.insert("userid".to_string(), vec![userid.to_string()]);
    let req: Request = req.into();
    req.with_path_parameters(QueryMap::from(params))
}

pub fn request_no_userid(method: &str, body: Option<&str>) -> Request {
    let b = match body {
        Some(t) => Body::Text(t.to_string()),
        None => Body::Empty,
    };
    let req = http::Request::builder()
        .method(method)
        .uri("/users")
        .body(b)
        .unwrap();
    req.into()
}

#[allow(dead_code)]
pub fn response_body_string(resp: &lambda_http::Response<Body>) -> String {
    match resp.body() {
        Body::Text(t) => t.clone(),
        Body::Empty => String::new(),
        _ => panic!("unexpected body type"),
    }
}

#[allow(dead_code)]
pub fn dynamo_conditional_check_failed() -> ReplayEvent {
    ReplayEvent::new(
        http::Request::builder()
            .body(SdkBody::from(""))
            .unwrap(),
        http::Response::builder()
            .status(400)
            .header("x-amzn-errortype", "ConditionalCheckFailedException")
            .header("content-type", "application/x-amz-json-1.0")
            .body(SdkBody::from(
                r#"{"__type":"ConditionalCheckFailedException","Message":"The conditional request failed"}"#,
            ))
            .unwrap(),
    )
}

#[allow(dead_code)]
pub const INTERNAL_SERVER_ERROR: &str = r#"{"__type":"InternalServerError","message":"Service Unavailable"}"#;
