output "users_table_arn" {
  description = "ARN of the users DynamoDB table"
  value       = component.workshop.users_table_arn
  type        = string
}

output "users_table_id" {
  description = "ID of the users DynamoDB table"
  value       = component.workshop.users_table_id
  type        = string
}

output "users_table_name" {
  description = "Name of the users DynamoDB table"
  value       = component.workshop.users_table_name
  type        = string
}

output "api_gateway_endpoint" {
  description = "HTTP API Gateway endpoint URL"
  value       = component.workshop.api_gateway_endpoint
  type        = string
}

output "api_gateway_execution_arn" {
  description = "Execution ARN of the HTTP API Gateway"
  value       = component.workshop.api_gateway_execution_arn
  type        = string
}

output "api_gateway_stage_url" {
  description = "Full stage invoke URL for the Prod stage"
  value       = component.workshop.api_gateway_stage_url
  type        = string
}

output "cognito_user_pool_id" {
  description = "ID of the Cognito User Pool"
  value       = component.workshop.cognito_user_pool_id
  type        = string
}

output "cognito_user_pool_client_id" {
  description = "ID of the Cognito User Pool Client"
  value       = component.workshop.cognito_user_pool_client_id
  type        = string
}

output "cognito_domain_url" {
  description = "Cognito hosted UI domain URL"
  value       = component.workshop.cognito_domain_url
  type        = string
}

output "cognito_login_url" {
  description = "Cognito login URL for the hosted UI"
  value       = component.workshop.cognito_login_url
  type        = string
}

output "authorizer_lambda_function_arn" {
  description = "ARN of the authorizer Lambda function"
  value       = component.workshop.authorizer_lambda_function_arn
  type        = string
}

output "authorizer_id" {
  description = "ID of the API Gateway authorizer"
  value       = component.workshop.authorizer_id
  type        = string
}
