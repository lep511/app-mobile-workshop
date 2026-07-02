use authorizer_lambda::{
    denied_response, extract_token, jwks_url, validate_token, Jwk, JwksResponse,
};
use base64::Engine;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rand::rngs::OsRng;
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use serde::Serialize;
use std::collections::HashMap;

const TEST_REGION: &str = "us-east-1";
const TEST_POOL_ID: &str = "us-east-1_TestPool";
const TEST_CLIENT_ID: &str = "test-client-id";
const TEST_KID: &str = "test-key-id";

#[derive(Serialize)]
struct TestClaims {
    sub: String,
    iss: String,
    client_id: String,
    token_use: String,
    username: String,
    exp: u64,
}

fn generate_test_keypair() -> (RsaPrivateKey, Jwk) {
    let private_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
    let public_key = private_key.to_public_key();

    let n_bytes = public_key.n().to_bytes_be();
    let e_bytes = public_key.e().to_bytes_be();

    let n_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&n_bytes);
    let e_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&e_bytes);

    let jwk = Jwk {
        kid: TEST_KID.to_string(),
        n: n_b64,
        e: e_b64,
    };

    (private_key, jwk)
}

fn make_valid_token(private_key: &RsaPrivateKey) -> String {
    let claims = TestClaims {
        sub: "user-sub-123".into(),
        iss: format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            TEST_REGION, TEST_POOL_ID
        ),
        client_id: TEST_CLIENT_ID.into(),
        token_use: "access".into(),
        username: "testuser".into(),
        exp: 9999999999,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(TEST_KID.to_string());

    let pem = private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .unwrap();
    let encoding_key = EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap();

    encode(&header, &claims, &encoding_key).unwrap()
}

// --- extract_token ---

#[test]
fn test_extract_token_lowercase_header() {
    let mut headers = HashMap::new();
    headers.insert("authorization".into(), "Bearer mytoken".into());
    assert_eq!(extract_token(&headers), Some("mytoken".into()));
}

#[test]
fn test_extract_token_capitalized_header() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".into(), "Bearer mytoken".into());
    assert_eq!(extract_token(&headers), Some("mytoken".into()));
}

#[test]
fn test_extract_token_no_bearer_prefix() {
    let mut headers = HashMap::new();
    headers.insert("authorization".into(), "rawtoken".into());
    assert_eq!(extract_token(&headers), Some("rawtoken".into()));
}

#[test]
fn test_extract_token_missing() {
    let headers = HashMap::new();
    assert_eq!(extract_token(&headers), None);
}

#[test]
fn test_extract_token_empty_value() {
    let mut headers = HashMap::new();
    headers.insert("authorization".into(), "".into());
    assert_eq!(extract_token(&headers), None);
}

// --- jwks_url ---

#[test]
fn test_jwks_url_format() {
    let url = jwks_url("us-west-2", "us-west-2_ABC123");
    assert_eq!(
        url,
        "https://cognito-idp.us-west-2.amazonaws.com/us-west-2_ABC123/.well-known/jwks.json"
    );
}

// --- denied_response ---

#[test]
fn test_denied_response() {
    let resp = denied_response();
    assert!(!resp.is_authorized);
    assert!(resp.context.user_id.is_empty());
    assert!(resp.context.username.is_empty());
}

// --- validate_token ---

#[test]
fn test_validate_token_valid() {
    let (private_key, jwk) = generate_test_keypair();
    let token = make_valid_token(&private_key);
    let jwks = JwksResponse { keys: vec![jwk] };

    let resp = validate_token(&token, &jwks, TEST_REGION, TEST_POOL_ID, TEST_CLIENT_ID);
    assert!(resp.is_authorized);
    assert_eq!(resp.context.user_id, "user-sub-123");
    assert_eq!(resp.context.username, "testuser");
}

#[test]
fn test_validate_token_invalid_header() {
    let jwks = JwksResponse { keys: vec![] };
    let resp = validate_token(
        "not.a.jwt",
        &jwks,
        TEST_REGION,
        TEST_POOL_ID,
        TEST_CLIENT_ID,
    );
    assert!(!resp.is_authorized);
}

#[test]
fn test_validate_token_garbage_input() {
    let jwks = JwksResponse { keys: vec![] };
    let resp = validate_token(
        "totalgarbage",
        &jwks,
        TEST_REGION,
        TEST_POOL_ID,
        TEST_CLIENT_ID,
    );
    assert!(!resp.is_authorized);
}

#[test]
fn test_validate_token_missing_kid() {
    let (private_key, jwk) = generate_test_keypair();
    let jwks = JwksResponse { keys: vec![jwk] };

    let claims = TestClaims {
        sub: "user-sub-123".into(),
        iss: format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            TEST_REGION, TEST_POOL_ID
        ),
        client_id: TEST_CLIENT_ID.into(),
        token_use: "access".into(),
        username: "testuser".into(),
        exp: 9999999999,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = None; // no kid

    let pem = private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .unwrap();
    let encoding_key = EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap();
    let token = encode(&header, &claims, &encoding_key).unwrap();

    let resp = validate_token(&token, &jwks, TEST_REGION, TEST_POOL_ID, TEST_CLIENT_ID);
    assert!(!resp.is_authorized);
}

#[test]
fn test_validate_token_kid_not_in_jwks() {
    let (private_key, _) = generate_test_keypair();
    let token = make_valid_token(&private_key);

    let other_jwk = Jwk {
        kid: "other-kid".into(),
        n: "AAAA".into(),
        e: "AQAB".into(),
    };
    let jwks = JwksResponse {
        keys: vec![other_jwk],
    };

    let resp = validate_token(&token, &jwks, TEST_REGION, TEST_POOL_ID, TEST_CLIENT_ID);
    assert!(!resp.is_authorized);
}

#[test]
fn test_validate_token_invalid_rsa_components() {
    let (private_key, _) = generate_test_keypair();
    let token = make_valid_token(&private_key);

    let bad_jwk = Jwk {
        kid: TEST_KID.into(),
        n: "not-valid-base64-rsa".into(),
        e: "also-bad".into(),
    };
    let jwks = JwksResponse {
        keys: vec![bad_jwk],
    };

    let resp = validate_token(&token, &jwks, TEST_REGION, TEST_POOL_ID, TEST_CLIENT_ID);
    assert!(!resp.is_authorized);
}

#[test]
fn test_validate_token_expired() {
    let (private_key, jwk) = generate_test_keypair();
    let jwks = JwksResponse { keys: vec![jwk] };

    let claims = TestClaims {
        sub: "user-sub-123".into(),
        iss: format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            TEST_REGION, TEST_POOL_ID
        ),
        client_id: TEST_CLIENT_ID.into(),
        token_use: "access".into(),
        username: "testuser".into(),
        exp: 1000000000, // expired in 2001
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(TEST_KID.to_string());

    let pem = private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .unwrap();
    let encoding_key = EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap();
    let token = encode(&header, &claims, &encoding_key).unwrap();

    let resp = validate_token(&token, &jwks, TEST_REGION, TEST_POOL_ID, TEST_CLIENT_ID);
    assert!(!resp.is_authorized);
}

#[test]
fn test_validate_token_wrong_client_id() {
    let (private_key, jwk) = generate_test_keypair();
    let token = make_valid_token(&private_key);
    let jwks = JwksResponse { keys: vec![jwk] };

    let resp = validate_token(&token, &jwks, TEST_REGION, TEST_POOL_ID, "wrong-client-id");
    assert!(!resp.is_authorized);
}

#[test]
fn test_validate_token_wrong_token_use() {
    let (private_key, jwk) = generate_test_keypair();
    let jwks = JwksResponse { keys: vec![jwk] };

    let claims = TestClaims {
        sub: "user-sub-123".into(),
        iss: format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            TEST_REGION, TEST_POOL_ID
        ),
        client_id: TEST_CLIENT_ID.into(),
        token_use: "id".into(), // wrong: should be "access"
        username: "testuser".into(),
        exp: 9999999999,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(TEST_KID.to_string());

    let pem = private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .unwrap();
    let encoding_key = EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap();
    let token = encode(&header, &claims, &encoding_key).unwrap();

    let resp = validate_token(&token, &jwks, TEST_REGION, TEST_POOL_ID, TEST_CLIENT_ID);
    assert!(!resp.is_authorized);
}

#[test]
fn test_validate_token_wrong_issuer() {
    let (private_key, jwk) = generate_test_keypair();
    let jwks = JwksResponse { keys: vec![jwk] };

    let claims = TestClaims {
        sub: "user-sub-123".into(),
        iss: "https://wrong-issuer.com".into(),
        client_id: TEST_CLIENT_ID.into(),
        token_use: "access".into(),
        username: "testuser".into(),
        exp: 9999999999,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(TEST_KID.to_string());

    let pem = private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .unwrap();
    let encoding_key = EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap();
    let token = encode(&header, &claims, &encoding_key).unwrap();

    let resp = validate_token(&token, &jwks, TEST_REGION, TEST_POOL_ID, TEST_CLIENT_ID);
    assert!(!resp.is_authorized);
}

#[test]
fn test_validate_token_signed_with_wrong_key() {
    let (_, jwk) = generate_test_keypair(); // public key from keypair 1
    let (other_key, _) = generate_test_keypair(); // sign with different key
    let jwks = JwksResponse { keys: vec![jwk] };

    let claims = TestClaims {
        sub: "user-sub-123".into(),
        iss: format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            TEST_REGION, TEST_POOL_ID
        ),
        client_id: TEST_CLIENT_ID.into(),
        token_use: "access".into(),
        username: "testuser".into(),
        exp: 9999999999,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(TEST_KID.to_string());

    let pem = other_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF).unwrap();
    let encoding_key = EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap();
    let token = encode(&header, &claims, &encoding_key).unwrap();

    let resp = validate_token(&token, &jwks, TEST_REGION, TEST_POOL_ID, TEST_CLIENT_ID);
    assert!(!resp.is_authorized);
}
