use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2AuthorizerEvent {
    pub headers: Option<HashMap<String, String>>,
    pub route_arn: Option<String>,
    pub raw_path: Option<String>,
    pub request_context: Option<RequestContext>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestContext {
    pub route_key: Option<String>,
    pub account_id: Option<String>,
    pub stage: Option<String>,
    pub api_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizerResponse {
    pub is_authorized: bool,
    pub context: AuthContext,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthContext {
    pub user_id: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct JwksResponse {
    pub keys: Vec<Jwk>,
}

#[derive(Deserialize)]
pub struct Jwk {
    pub kid: String,
    pub n: String,
    pub e: String,
}

#[derive(Deserialize)]
pub struct TokenClaims {
    pub sub: Option<String>,
    #[allow(dead_code)]
    pub iss: Option<String>,
    pub client_id: Option<String>,
    pub token_use: Option<String>,
    pub username: Option<String>,
}

pub fn denied_response() -> AuthorizerResponse {
    AuthorizerResponse {
        is_authorized: false,
        context: AuthContext {
            user_id: String::new(),
            username: String::new(),
        },
    }
}

pub fn extract_token(headers: &HashMap<String, String>) -> Option<String> {
    let auth_value = headers
        .get("authorization")
        .or_else(|| headers.get("Authorization"))?;

    if auth_value.is_empty() {
        return None;
    }

    Some(auth_value.trim_start_matches("Bearer ").to_string())
}

pub fn jwks_url(region: &str, user_pool_id: &str) -> String {
    format!(
        "https://cognito-idp.{}.amazonaws.com/{}/.well-known/jwks.json",
        region, user_pool_id
    )
}

pub fn validate_token(
    token: &str,
    jwks: &JwksResponse,
    region: &str,
    user_pool_id: &str,
    client_id: &str,
) -> AuthorizerResponse {
    let header = match jsonwebtoken::decode_header(token) {
        Ok(h) => h,
        Err(_) => return denied_response(),
    };

    let kid = match header.kid {
        Some(k) => k,
        None => return denied_response(),
    };

    let jwk = match jwks.keys.iter().find(|k| k.kid == kid) {
        Some(k) => k,
        None => return denied_response(),
    };

    let decoding_key = match jsonwebtoken::DecodingKey::from_rsa_components(&jwk.n, &jwk.e) {
        Ok(k) => k,
        Err(_) => return denied_response(),
    };

    let expected_issuer = format!(
        "https://cognito-idp.{}.amazonaws.com/{}",
        region, user_pool_id
    );

    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_issuer(&[&expected_issuer]);
    validation.validate_exp = true;
    validation.set_required_spec_claims(&["exp", "iss", "sub"]);

    let token_data = match jsonwebtoken::decode::<TokenClaims>(token, &decoding_key, &validation) {
        Ok(d) => d,
        Err(_) => return denied_response(),
    };

    let claims = token_data.claims;

    if claims.client_id.as_deref() != Some(client_id) {
        return denied_response();
    }

    if claims.token_use.as_deref() != Some("access") {
        return denied_response();
    }

    let sub = claims.sub.unwrap_or_default();
    let username = claims.username.unwrap_or_default();

    AuthorizerResponse {
        is_authorized: true,
        context: AuthContext {
            user_id: sub,
            username,
        },
    }
}
