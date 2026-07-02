use authorizer_lambda::{
    denied_response, extract_token, jwks_url, validate_token, ApiGatewayV2AuthorizerEvent,
    AuthorizerResponse, JwksResponse,
};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use std::env;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .without_time()
        .with_target(true)
        .init();

    lambda_runtime::run(service_fn(handler)).await
}

async fn handler(
    event: LambdaEvent<ApiGatewayV2AuthorizerEvent>,
) -> Result<AuthorizerResponse, Error> {
    let (event, context) = event.into_parts();
    let request_id = &context.request_id;

    let route_arn = event.route_arn.as_deref().unwrap_or("unknown");
    info!(request_id = %request_id, route_arn = %route_arn, "Authorizer invoked");

    let headers = event.headers.unwrap_or_default();

    let token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            warn!(request_id = %request_id, "Missing or empty Authorization header");
            return Ok(denied_response());
        }
    };

    let region = env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let user_pool_id = env::var("USER_POOL_ID")?;
    let client_id = env::var("CLIENT_ID")?;

    let url = jwks_url(&region, &user_pool_id);
    let jwks: JwksResponse = match reqwest::get(&url).await {
        Ok(resp) => match resp.json().await {
            Ok(j) => j,
            Err(e) => {
                error!(request_id = %request_id, error = %e, "Failed to parse JWKS response");
                return Err(e.into());
            }
        },
        Err(e) => {
            error!(request_id = %request_id, url = %url, error = %e, "Failed to fetch JWKS");
            return Err(e.into());
        }
    };

    let response = validate_token(&token, &jwks, &region, &user_pool_id, &client_id);

    if response.is_authorized {
        info!(
            request_id = %request_id,
            user_id = %response.context.user_id,
            "Authorization granted"
        );
    } else {
        warn!(request_id = %request_id, "Authorization denied");
    }

    Ok(response)
}
