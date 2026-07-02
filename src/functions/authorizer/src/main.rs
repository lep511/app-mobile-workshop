use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use tracing::error;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiGatewayV2AuthorizerEvent {
    headers: Option<HashMap<String, String>>,
    route_arn: Option<String>,
    raw_path: Option<String>,
    request_context: Option<RequestContext>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestContext {
    route_key: Option<String>,
    account_id: Option<String>,
    stage: Option<String>,
    api_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorizerResponse {
    is_authorized: bool,
    context: AuthContext,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthContext {
    user_id: String,
    username: String,
}

#[derive(Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

#[derive(Deserialize)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
}

#[derive(Deserialize)]
struct TokenClaims {
    sub: Option<String>,
    #[allow(dead_code)]
    iss: Option<String>,
    client_id: Option<String>,
    token_use: Option<String>,
    username: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .without_time()
        .init();

    lambda_runtime::run(service_fn(handler)).await
}

async fn handler(event: LambdaEvent<ApiGatewayV2AuthorizerEvent>) -> Result<AuthorizerResponse, Error> {
    let (event, _context) = event.into_parts();

    let denied = AuthorizerResponse {
        is_authorized: false,
        context: AuthContext {
            user_id: String::new(),
            username: String::new(),
        },
    };

    let headers = event.headers.unwrap_or_default();
    let auth_header = headers
        .get("authorization")
        .or_else(|| headers.get("Authorization"))
        .map(|s| s.as_str())
        .unwrap_or("");

    if auth_header.is_empty() {
        error!("Missing Authorization header");
        return Ok(denied);
    }

    let token = auth_header.trim_start_matches("Bearer ").to_string();

    let region = env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let user_pool_id = env::var("USER_POOL_ID")?;
    let client_id = env::var("CLIENT_ID")?;

    let jwks_url = format!(
        "https://cognito-idp.{}.amazonaws.com/{}/.well-known/jwks.json",
        region, user_pool_id
    );

    let jwks: JwksResponse = reqwest::get(&jwks_url).await?.json().await?;

    let header = match jsonwebtoken::decode_header(&token) {
        Ok(h) => h,
        Err(e) => {
            error!("Failed to decode token header: {}", e);
            return Ok(denied);
        }
    };

    let kid = match header.kid {
        Some(k) => k,
        None => {
            error!("Token header missing kid");
            return Ok(denied);
        }
    };

    let jwk = match jwks.keys.iter().find(|k| k.kid == kid) {
        Some(k) => k,
        None => {
            error!("Key not found in JWKS");
            return Ok(denied);
        }
    };

    let decoding_key = match jsonwebtoken::DecodingKey::from_rsa_components(&jwk.n, &jwk.e) {
        Ok(k) => k,
        Err(e) => {
            error!("Failed to build decoding key: {}", e);
            return Ok(denied);
        }
    };

    let expected_issuer = format!(
        "https://cognito-idp.{}.amazonaws.com/{}",
        region, user_pool_id
    );

    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_issuer(&[&expected_issuer]);
    validation.validate_exp = true;
    validation.set_required_spec_claims(&["exp", "iss", "sub"]);

    let token_data = match jsonwebtoken::decode::<TokenClaims>(&token, &decoding_key, &validation) {
        Ok(d) => d,
        Err(e) => {
            error!("Token validation failed: {}", e);
            return Ok(denied);
        }
    };

    let claims = token_data.claims;

    if claims.client_id.as_deref() != Some(&client_id) {
        error!("Invalid client_id");
        return Ok(denied);
    }

    if claims.token_use.as_deref() != Some("access") {
        error!("Invalid token_use");
        return Ok(denied);
    }

    let sub = claims.sub.unwrap_or_default();
    let username = claims.username.unwrap_or_default();

    Ok(AuthorizerResponse {
        is_authorized: true,
        context: AuthContext {
            user_id: sub,
            username,
        },
    })
}
