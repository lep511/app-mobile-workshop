variable "region" {
  description = "AWS region for resource deployment"
  type        = string
  default     = "us-west-2"
}

variable "role_arn" {
  description = "ARN of the IAM role to assume via OIDC"
  type        = string
}

variable "identity_token" {
  description = "JWT identity token for OIDC authentication"
  type        = string
  ephemeral   = true
}

variable "workshop_stack_base_name" {
  description = "Base name for the workshop stack"
  type        = string
  default     = "workshop"
}

variable "environment" {
  description = "Environment name"
  type        = string
}

variable "project" {
  description = "Project name"
  type        = string
  default     = "Serverless Patterns"
}

variable "cors_allowed_origins" {
  description = "Allowed origins for CORS on the API Gateway"
  type        = list(string)
  default     = ["*"]
}
