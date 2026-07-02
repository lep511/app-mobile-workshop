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
    let has_userid = request
        .path_parameters_ref()
        .and_then(|p| p.first("userid"))
        .is_some_and(|v| !v.is_empty());

    tracing::info!(method = %method, path = %path, "Handling request");

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
        Ok(response) => Ok(response),
        Err(e) => {
            match &e {
                AppError::NotFound(_) => tracing::warn!(error = %e, "Resource not found"),
                AppError::ValidationError(_) => tracing::warn!(error = %e, "Validation error"),
                _ => tracing::error!(error = %e, "Request failed"),
            }
            Ok(e.into_response())
        }
    }
}
