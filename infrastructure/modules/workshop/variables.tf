variable "workshop_stack_base_name" {
  description = "Base name for the workshop stack"
  type        = string
}

variable "environment" {
  description = "Environment name"
  type        = string
}

variable "project" {
  description = "Project name"
  type        = string
}


variable "cors_allowed_origins" {
  description = "Allowed origins for CORS on the API Gateway"
  type        = list(string)
  default     = ["*"]
}

variable "lambda_log_level" {
  description = "Log level for Lambda functions (RUST_LOG format, e.g. info, debug, warn)"
  type        = string
  default     = "info"
}
