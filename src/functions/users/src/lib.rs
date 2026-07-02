use aws_sdk_dynamodb::Client as DynamoClient;
use lambda_http::{Body, Error, Request, RequestExt, Response};
use std::env;

pub mod errors;
pub mod handlers;
pub mod models;

use errors::AppError;

pub struct AppState {
    pub dynamo_client: DynamoClient,
    pub table_name: String,
}

impl AppState {
    pub fn new(dynamo_client: DynamoClient, table_name: String) -> Self {
        Self {
            dynamo_client,
            table_name,
        }
    }

    pub fn from_env(dynamo_client: DynamoClient) -> Self {
        let table_name = env::var("USERS_TABLE_NAME")
            .expect("USERS_TABLE_NAME environment variable must be set");
        Self::new(dynamo_client, table_name)
    }
}

pub async fn handle_request(request: Request, state: &AppState) -> Result<Response<Body>, Error> {
    let method = request.method().as_str().to_uppercase();
    let path = request.uri().path().to_string();
    let request_id = request
        .lambda_context_ref()
        .map(|ctx| ctx.request_id.as_str())
        .unwrap_or("unknown");
    let has_userid = request
        .path_parameters_ref()
        .and_then(|p| p.first("userid"))
        .is_some_and(|v| !v.is_empty());

    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        "Handling request"
    );

    let result = match method.as_str() {
        "GET" if has_userid => handlers::get_user(state, &request).await,
        "GET" => handlers::list_users(state, &request).await,
        "PUT" if has_userid => handlers::update_user(state, &request).await,
        "PUT" => handlers::create_user(state, &request).await,
        "DELETE" if has_userid => handlers::delete_user(state, &request).await,
        "OPTIONS" => Ok(handlers::options_response()),
        _ => Err(AppError::MethodNotAllowed),
    };

    match result {
        Ok(response) => {
            tracing::info!(
                request_id = %request_id,
                status = response.status().as_u16(),
                "Request completed"
            );
            Ok(response)
        }
        Err(e) => {
            let status = e.status_code();
            match &e {
                AppError::NotFound(_) => tracing::warn!(
                    request_id = %request_id, status, error = %e, "Resource not found"
                ),
                AppError::ValidationError(_) => tracing::warn!(
                    request_id = %request_id, status, error = %e, "Validation error"
                ),
                AppError::Conflict(_) => tracing::warn!(
                    request_id = %request_id, status, error = %e, "Conflict"
                ),
                AppError::MethodNotAllowed => tracing::warn!(
                    request_id = %request_id, status, method = %method, "Method not allowed"
                ),
                AppError::Internal(_) => tracing::error!(
                    request_id = %request_id, status, error = %e, "Internal error"
                ),
            }
            Ok(e.into_response())
        }
    }
}
