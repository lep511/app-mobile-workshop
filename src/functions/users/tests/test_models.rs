use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;
use users_lambda::errors::AppError;
use users_lambda::models::{CreateUserRequest, UpdateUserRequest, User};

// --- CreateUserRequest::validate ---

#[test]
fn test_create_valid_request() {
    let req = CreateUserRequest {
        email: "user@example.com".into(),
        name: "Test User".into(),
        phone: Some("+1234567890".into()),
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_create_valid_without_phone() {
    let req = CreateUserRequest {
        email: "user@example.com".into(),
        name: "Test User".into(),
        phone: None,
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_create_empty_email() {
    let req = CreateUserRequest {
        email: "   ".into(),
        name: "Test".into(),
        phone: None,
    };
    let err = req.validate().unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg.contains("email is required")));
}

#[test]
fn test_create_email_too_long() {
    let req = CreateUserRequest {
        email: "a".repeat(255),
        name: "Test".into(),
        phone: None,
    };
    let err = req.validate().unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg.contains("254")));
}

#[test]
fn test_create_empty_name() {
    let req = CreateUserRequest {
        email: "user@example.com".into(),
        name: "  ".into(),
        phone: None,
    };
    let err = req.validate().unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg.contains("name is required")));
}

#[test]
fn test_create_name_too_long() {
    let req = CreateUserRequest {
        email: "user@example.com".into(),
        name: "a".repeat(257),
        phone: None,
    };
    let err = req.validate().unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg.contains("256")));
}

#[test]
fn test_create_phone_too_long() {
    let req = CreateUserRequest {
        email: "user@example.com".into(),
        name: "Test".into(),
        phone: Some("a".repeat(21)),
    };
    let err = req.validate().unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg.contains("20")));
}

// --- UpdateUserRequest::validate ---

#[test]
fn test_update_all_none_fails() {
    let req = UpdateUserRequest {
        email: None,
        name: None,
        phone: None,
    };
    let err = req.validate().unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg.contains("at least one field")));
}

#[test]
fn test_update_email_only_valid() {
    let req = UpdateUserRequest {
        email: Some("new@example.com".into()),
        name: None,
        phone: None,
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_update_email_empty_fails() {
    let req = UpdateUserRequest {
        email: Some("  ".into()),
        name: None,
        phone: None,
    };
    let err = req.validate().unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg.contains("email cannot be empty")));
}

#[test]
fn test_update_email_too_long() {
    let req = UpdateUserRequest {
        email: Some("a".repeat(255)),
        name: None,
        phone: None,
    };
    let err = req.validate().unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg.contains("254")));
}

#[test]
fn test_update_name_empty_fails() {
    let req = UpdateUserRequest {
        email: None,
        name: Some("  ".into()),
        phone: None,
    };
    let err = req.validate().unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg.contains("name cannot be empty")));
}

#[test]
fn test_update_name_too_long() {
    let req = UpdateUserRequest {
        email: None,
        name: Some("a".repeat(257)),
        phone: None,
    };
    let err = req.validate().unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg.contains("256")));
}

#[test]
fn test_update_phone_too_long() {
    let req = UpdateUserRequest {
        email: None,
        name: None,
        phone: Some("a".repeat(21)),
    };
    let err = req.validate().unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg.contains("20")));
}

#[test]
fn test_update_phone_only_valid() {
    let req = UpdateUserRequest {
        email: None,
        name: None,
        phone: Some("+1234567890".into()),
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_update_all_fields_valid() {
    let req = UpdateUserRequest {
        email: Some("new@example.com".into()),
        name: Some("New Name".into()),
        phone: Some("+9876543210".into()),
    };
    assert!(req.validate().is_ok());
}

// --- User::from_dynamodb_item ---

#[test]
fn test_from_item_all_fields() {
    let mut item = HashMap::new();
    item.insert("userid".into(), AttributeValue::S("u1".into()));
    item.insert("email".into(), AttributeValue::S("a@b.com".into()));
    item.insert("name".into(), AttributeValue::S("Alice".into()));
    item.insert("phone".into(), AttributeValue::S("+123".into()));

    let user = User::from_dynamodb_item(&item).unwrap();
    assert_eq!(user.userid, "u1");
    assert_eq!(user.email, "a@b.com");
    assert_eq!(user.name, "Alice");
    assert_eq!(user.phone.as_deref(), Some("+123"));
}

#[test]
fn test_from_item_without_phone() {
    let mut item = HashMap::new();
    item.insert("userid".into(), AttributeValue::S("u1".into()));
    item.insert("email".into(), AttributeValue::S("a@b.com".into()));
    item.insert("name".into(), AttributeValue::S("Alice".into()));

    let user = User::from_dynamodb_item(&item).unwrap();
    assert!(user.phone.is_none());
}

#[test]
fn test_from_item_missing_userid() {
    let mut item = HashMap::new();
    item.insert("email".into(), AttributeValue::S("a@b.com".into()));
    item.insert("name".into(), AttributeValue::S("Alice".into()));

    let err = User::from_dynamodb_item(&item).unwrap_err();
    assert!(matches!(err, AppError::Internal(msg) if msg.contains("userid")));
}

#[test]
fn test_from_item_missing_email() {
    let mut item = HashMap::new();
    item.insert("userid".into(), AttributeValue::S("u1".into()));
    item.insert("name".into(), AttributeValue::S("Alice".into()));

    let err = User::from_dynamodb_item(&item).unwrap_err();
    assert!(matches!(err, AppError::Internal(msg) if msg.contains("email")));
}

#[test]
fn test_from_item_missing_name() {
    let mut item = HashMap::new();
    item.insert("userid".into(), AttributeValue::S("u1".into()));
    item.insert("email".into(), AttributeValue::S("a@b.com".into()));

    let err = User::from_dynamodb_item(&item).unwrap_err();
    assert!(matches!(err, AppError::Internal(msg) if msg.contains("name")));
}

// --- User::to_dynamodb_item ---

#[test]
fn test_to_item_with_phone() {
    let user = User {
        userid: "u1".into(),
        email: "a@b.com".into(),
        name: "Alice".into(),
        phone: Some("+123".into()),
    };
    let item = user.to_dynamodb_item();
    assert_eq!(item.len(), 4);
    assert_eq!(item["userid"].as_s().unwrap(), "u1");
    assert_eq!(item["phone"].as_s().unwrap(), "+123");
}

#[test]
fn test_to_item_without_phone() {
    let user = User {
        userid: "u1".into(),
        email: "a@b.com".into(),
        name: "Alice".into(),
        phone: None,
    };
    let item = user.to_dynamodb_item();
    assert_eq!(item.len(), 3);
    assert!(!item.contains_key("phone"));
}
