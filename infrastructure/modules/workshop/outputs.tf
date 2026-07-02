output "users_table_arn" {
  description = "ARN of the users DynamoDB table"
  value       = aws_dynamodb_table.users_table.arn
}

output "users_table_id" {
  description = "ID of the users DynamoDB table"
  value       = aws_dynamodb_table.users_table.id
}

output "users_table_name" {
  description = "Name of the users DynamoDB table"
  value       = aws_dynamodb_table.users_table.name
}

output "users_lambda_function_name" {
  description = "Name of the users Lambda function"
  value       = aws_lambda_function.users.function_name
}

output "users_lambda_function_arn" {
  description = "ARN of the users Lambda function"
  value       = aws_lambda_function.users.arn
}

output "api_gateway_endpoint" {
  description = "HTTP API Gateway endpoint URL"
  value       = aws_apigatewayv2_api.users.api_endpoint
}

output "api_gateway_execution_arn" {
  description = "Execution ARN of the HTTP API Gateway"
  value       = aws_apigatewayv2_api.users.execution_arn
}

output "api_gateway_stage_url" {
  description = "Full stage invoke URL for the Prod stage"
  value       = aws_apigatewayv2_stage.prod.invoke_url
}

output "cognito_user_pool_id" {
  description = "ID of the Cognito User Pool"
  value       = aws_cognito_user_pool.main.id
}

output "cognito_user_pool_client_id" {
  description = "ID of the Cognito User Pool Client"
  value       = aws_cognito_user_pool_client.main.id
}

output "cognito_domain_url" {
  description = "Cognito hosted UI domain URL"
  value       = "https://${aws_cognito_user_pool_domain.main.domain}.auth.${data.aws_region.current.region}.amazoncognito.com"
}

output "cognito_login_url" {
  description = "Cognito login URL for the hosted UI"
  value       = "https://${aws_cognito_user_pool_domain.main.domain}.auth.${data.aws_region.current.region}.amazoncognito.com/login?client_id=${aws_cognito_user_pool_client.main.id}&response_type=code&scope=email+openid&redirect_uri=http://localhost"
}

output "authorizer_lambda_function_arn" {
  description = "ARN of the authorizer Lambda function"
  value       = aws_lambda_function.authorizer.arn
}

output "authorizer_id" {
  description = "ID of the API Gateway authorizer"
  value       = aws_apigatewayv2_authorizer.cognito.id
}

