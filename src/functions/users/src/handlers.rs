use aws_sdk_dynamodb::operation::delete_item::DeleteItemError;
use aws_sdk_dynamodb::operation::put_item::PutItemError;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_http::{Body, Request, RequestExt, Response};
use tracing::info;
use uuid::Uuid;

use crate::errors::{success_response, AppError};
use crate::models::{CreateUserRequest, ListUsersResponse, UpdateUserRequest, User};
use crate::AppState;

const DEFAULT_PAGE_SIZE: i32 = 20;
const MAX_PAGE_SIZE: i32 = 100;

pub async fn list_users(state: &AppState, request: &Request) -> Result<Response<Body>, AppError> {
    let limit = request
        .query_string_parameters_ref()
        .and_then(|p| p.first("limit"))
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);

    let mut scan_builder = state
        .dynamo_client
        .scan()
        .table_name(&state.table_name)
        .limit(limit);

    if let Some(next_token) = request
        .query_string_parameters_ref()
        .and_then(|p| p.first("next_token"))
    {
        let decoded = decode_pagination_token(next_token)?;
        scan_builder = scan_builder.exclusive_start_key("userid", AttributeValue::S(decoded));
    }

    let result = scan_builder
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("DynamoDB scan failed: {e}")))?;

    let users: Vec<User> = result
        .items()
        .iter()
        .filter_map(|item| User::from_dynamodb_item(item).ok())
        .collect();

    let next_token = result.last_evaluated_key().and_then(|key| {
        key.get("userid")
            .and_then(|v| v.as_s().ok())
            .map(|userid| encode_pagination_token(userid))
    });

    let response_body = ListUsersResponse { users, next_token };

    Ok(success_response(200, &response_body))
}

pub async fn get_user(state: &AppState, request: &Request) -> Result<Response<Body>, AppError> {
    let userid = extract_userid(request)?;

    let result = state
        .dynamo_client
        .get_item()
        .table_name(&state.table_name)
        .key("userid", AttributeValue::S(userid.clone()))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("DynamoDB get failed: {e}")))?;

    let item = result
        .item()
        .ok_or_else(|| AppError::NotFound(format!("User {userid} not found")))?;

    let user = User::from_dynamodb_item(item)?;

    Ok(success_response(200, &user))
}

pub async fn create_user(state: &AppState, request: &Request) -> Result<Response<Body>, AppError> {
    let body = match request.body() {
        Body::Text(text) => text.clone(),
        Body::Binary(bytes) => String::from_utf8(bytes.to_vec())
            .map_err(|_| AppError::ValidationError("invalid UTF-8 in request body".into()))?,
        Body::Empty => return Err(AppError::ValidationError("request body is required".into())),
        _ => {
            return Err(AppError::ValidationError(
                "unsupported request body encoding".into(),
            ))
        }
    };

    let create_req: CreateUserRequest = serde_json::from_str(&body)
        .map_err(|e| AppError::ValidationError(format!("invalid JSON: {e}")))?;

    create_req.validate()?;

    let userid = Uuid::new_v4().to_string();

    let user = User {
        userid: userid.clone(),
        email: create_req.email,
        name: create_req.name,
        phone: create_req.phone,
    };

    let item = user.to_dynamodb_item();

    state
        .dynamo_client
        .put_item()
        .table_name(&state.table_name)
        .set_item(Some(item))
        .condition_expression("attribute_not_exists(userid)")
        .send()
        .await
        .map_err(|e| match e.into_service_error() {
            PutItemError::ConditionalCheckFailedException(_) => {
                AppError::Conflict("user already exists".into())
            }
            other => AppError::Internal(format!("DynamoDB put failed: {other}")),
        })?;

    info!(userid = %userid, "User created");

    Ok(success_response(201, &user))
}

pub async fn update_user(state: &AppState, request: &Request) -> Result<Response<Body>, AppError> {
    let userid = extract_userid(request)?;

    let body = match request.body() {
        Body::Text(text) => text.clone(),
        Body::Binary(bytes) => String::from_utf8(bytes.to_vec())
            .map_err(|_| AppError::ValidationError("invalid UTF-8 in request body".into()))?,
        Body::Empty => return Err(AppError::ValidationError("request body is required".into())),
        _ => {
            return Err(AppError::ValidationError(
                "unsupported request body encoding".into(),
            ))
        }
    };

    let update_req: UpdateUserRequest = serde_json::from_str(&body)
        .map_err(|e| AppError::ValidationError(format!("invalid JSON: {e}")))?;

    update_req.validate()?;

    let mut update_parts: Vec<String> = Vec::new();
    let mut expr_names: Vec<(String, String)> = Vec::new();
    let mut expr_values: Vec<(String, AttributeValue)> = Vec::new();

    if let Some(ref email) = update_req.email {
        update_parts.push("#email = :email".into());
        expr_names.push(("#email".into(), "email".into()));
        expr_values.push((":email".into(), AttributeValue::S(email.clone())));
    }

    if let Some(ref name) = update_req.name {
        update_parts.push("#name = :name".into());
        expr_names.push(("#name".into(), "name".into()));
        expr_values.push((":name".into(), AttributeValue::S(name.clone())));
    }

    if let Some(ref phone) = update_req.phone {
        update_parts.push("#phone = :phone".into());
        expr_names.push(("#phone".into(), "phone".into()));
        expr_values.push((":phone".into(), AttributeValue::S(phone.clone())));
    }

    let update_expression = format!("SET {}", update_parts.join(", "));

    let mut update_builder = state
        .dynamo_client
        .update_item()
        .table_name(&state.table_name)
        .key("userid", AttributeValue::S(userid.clone()))
        .update_expression(&update_expression)
        .condition_expression("attribute_exists(userid)")
        .return_values(aws_sdk_dynamodb::types::ReturnValue::AllNew);

    for (name, value) in &expr_names {
        update_builder = update_builder.expression_attribute_names(name, value);
    }

    for (name, value) in expr_values {
        update_builder = update_builder.expression_attribute_values(name, value);
    }

    let result = update_builder
        .send()
        .await
        .map_err(|e| match e.into_service_error() {
            UpdateItemError::ConditionalCheckFailedException(_) => {
                AppError::NotFound(format!("User {userid} not found"))
            }
            other => AppError::Internal(format!("DynamoDB update failed: {other}")),
        })?;

    let attributes = result
        .attributes()
        .ok_or_else(|| AppError::Internal("no attributes returned from update".into()))?;

    let user = User::from_dynamodb_item(attributes)?;

    info!(userid = %userid, "User updated");

    Ok(success_response(200, &user))
}

pub async fn delete_user(state: &AppState, request: &Request) -> Result<Response<Body>, AppError> {
    let userid = extract_userid(request)?;

    state
        .dynamo_client
        .delete_item()
        .table_name(&state.table_name)
        .key("userid", AttributeValue::S(userid.clone()))
        .condition_expression("attribute_exists(userid)")
        .send()
        .await
        .map_err(|e| match e.into_service_error() {
            DeleteItemError::ConditionalCheckFailedException(_) => {
                AppError::NotFound(format!("User {userid} not found"))
            }
            other => AppError::Internal(format!("DynamoDB delete failed: {other}")),
        })?;

    info!(userid = %userid, "User deleted");

    Ok(Response::builder()
        .status(204)
        .header("content-type", "application/json")
        .body(Body::Empty)
        .unwrap())
}

pub fn options_response() -> Response<Body> {
    Response::builder()
        .status(200)
        .header("access-control-allow-origin", "*")
        .header("access-control-allow-methods", "GET, PUT, DELETE, OPTIONS")
        .header(
            "access-control-allow-headers",
            "content-type, authorization, x-amz-date, x-api-key",
        )
        .header("access-control-max-age", "3600")
        .body(Body::Empty)
        .unwrap()
}

pub fn extract_userid(request: &Request) -> Result<String, AppError> {
    request
        .path_parameters_ref()
        .and_then(|p| p.first("userid"))
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string())
        .ok_or_else(|| AppError::ValidationError("userid path parameter is required".into()))
}

pub fn encode_pagination_token(userid: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(userid.as_bytes())
}

pub fn decode_pagination_token(token: &str) -> Result<String, AppError> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| AppError::ValidationError("invalid pagination token".into()))?;
    String::from_utf8(bytes)
        .map_err(|_| AppError::ValidationError("invalid pagination token encoding".into()))
}
