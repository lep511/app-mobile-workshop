resource "aws_cognito_user_pool" "main" {
  name = "${var.workshop_stack_base_name}_UserPool"

  username_attributes      = ["email"]
  auto_verified_attributes = ["email"]

  admin_create_user_config {
    allow_admin_create_user_only = false
  }

  password_policy {
    minimum_length                   = 8
    require_numbers                  = true
    require_symbols                  = true
    require_uppercase                = true
    require_lowercase                = true
    temporary_password_validity_days = 7
  }

  account_recovery_setting {
    recovery_mechanism {
      name     = "verified_email"
      priority = 1
    }
  }

  user_pool_add_ons {
    advanced_security_mode = "AUDIT"
  }

  schema {
    name                = "email"
    attribute_data_type = "String"
    required            = true
    mutable             = true

    string_attribute_constraints {
      min_length = 1
      max_length = 256
    }
  }

  tags = {
    Name        = "${var.workshop_stack_base_name}-cognito-user-pool"
    Environment = var.environment
    Project     = var.project
  }
}

resource "aws_cognito_user_pool_client" "main" {
  name         = "${var.workshop_stack_base_name}-client"
  user_pool_id = aws_cognito_user_pool.main.id

  explicit_auth_flows = [
    "ALLOW_USER_PASSWORD_AUTH",
    "ALLOW_USER_SRP_AUTH",
    "ALLOW_REFRESH_TOKEN_AUTH",
  ]

  refresh_token_validity = 30

  callback_urls                = ["http://localhost"]
  allowed_oauth_flows          = ["code", "implicit"]
  allowed_oauth_scopes         = ["email", "openid"]
  allowed_oauth_flows_user_pool_client = true
  supported_identity_providers = ["COGNITO"]
}

resource "aws_cognito_user_pool_domain" "main" {
  domain       = var.workshop_stack_base_name
  user_pool_id = aws_cognito_user_pool.main.id
}

resource "aws_cognito_user_group" "administrators" {
  name         = "Administrators"
  user_pool_id = aws_cognito_user_pool.main.id
  description  = "Administrator group with elevated privileges"
}
