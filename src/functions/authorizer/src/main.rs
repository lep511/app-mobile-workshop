use authorizer_lambda::{
    denied_response, extract_token, jwks_url, validate_token, ApiGatewayV2AuthorizerEvent,
    AuthorizerResponse, JwksResponse,
};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use std::env;
use tracing::error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .without_time()
        .init();

    lambda_runtime::run(service_fn(handler)).await
}

async fn handler(
    event: LambdaEvent<ApiGatewayV2AuthorizerEvent>,
) -> Result<AuthorizerResponse, Error> {
    let (event, _context) = event.into_parts();

    let headers = event.headers.unwrap_or_default();

    let token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            error!("Missing Authorization header");
            return Ok(denied_response());
        }
    };

    let region = env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let user_pool_id = env::var("USER_POOL_ID")?;
    let client_id = env::var("CLIENT_ID")?;

    let url = jwks_url(&region, &user_pool_id);
    let jwks: JwksResponse = reqwest::get(&url).await?.json().await?;

    Ok(validate_token(
        &token,
        &jwks,
        &region,
        &user_pool_id,
        &client_id,
    ))
}
