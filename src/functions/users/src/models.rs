use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub userid: String,
    pub email: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    #[serde(default)]
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListUsersResponse {
    pub users: Vec<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

impl CreateUserRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.email.trim().is_empty() {
            return Err(AppError::ValidationError("email is required".into()));
        }
        if self.email.len() > 254 {
            return Err(AppError::ValidationError(
                "email exceeds maximum length of 254 characters".into(),
            ));
        }
        if self.name.trim().is_empty() {
            return Err(AppError::ValidationError("name is required".into()));
        }
        if self.name.len() > 256 {
            return Err(AppError::ValidationError(
                "name exceeds maximum length of 256 characters".into(),
            ));
        }
        if let Some(ref phone) = self.phone {
            if phone.len() > 20 {
                return Err(AppError::ValidationError(
                    "phone exceeds maximum length of 20 characters".into(),
                ));
            }
        }
        Ok(())
    }
}

impl UpdateUserRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.email.is_none() && self.name.is_none() && self.phone.is_none() {
            return Err(AppError::ValidationError(
                "at least one field must be provided for update".into(),
            ));
        }
        if let Some(ref email) = self.email {
            if email.trim().is_empty() {
                return Err(AppError::ValidationError("email cannot be empty".into()));
            }
            if email.len() > 254 {
                return Err(AppError::ValidationError(
                    "email exceeds maximum length of 254 characters".into(),
                ));
            }
        }
        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                return Err(AppError::ValidationError("name cannot be empty".into()));
            }
            if name.len() > 256 {
                return Err(AppError::ValidationError(
                    "name exceeds maximum length of 256 characters".into(),
                ));
            }
        }
        if let Some(ref phone) = self.phone {
            if phone.len() > 20 {
                return Err(AppError::ValidationError(
                    "phone exceeds maximum length of 20 characters".into(),
                ));
            }
        }
        Ok(())
    }
}

impl User {
    pub fn from_dynamodb_item(item: &HashMap<String, AttributeValue>) -> Result<Self, AppError> {
        let userid = item
            .get("userid")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| AppError::Internal("missing userid attribute".into()))?
            .clone();

        let email = item
            .get("email")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| AppError::Internal("missing email attribute".into()))?
            .clone();

        let name = item
            .get("name")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| AppError::Internal("missing name attribute".into()))?
            .clone();

        let phone = item.get("phone").and_then(|v| v.as_s().ok()).cloned();

        Ok(User {
            userid,
            email,
            name,
            phone,
        })
    }

    pub fn to_dynamodb_item(&self) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        item.insert("userid".into(), AttributeValue::S(self.userid.clone()));
        item.insert("email".into(), AttributeValue::S(self.email.clone()));
        item.insert("name".into(), AttributeValue::S(self.name.clone()));
        if let Some(ref phone) = self.phone {
            item.insert("phone".into(), AttributeValue::S(phone.clone()));
        }
        item
    }
}
