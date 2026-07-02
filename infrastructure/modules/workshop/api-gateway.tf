resource "aws_apigatewayv2_api" "users" {
  name          = "${var.workshop_stack_base_name}-users-api"
  protocol_type = "HTTP"

  cors_configuration {
    allow_headers  = ["content-type", "authorization", "x-amz-date", "x-api-key"]
    allow_methods  = ["GET", "PUT", "DELETE", "OPTIONS"]
    allow_origins  = var.cors_allowed_origins
    expose_headers = ["x-request-id", "x-amzn-requestid"]
    max_age        = 3600
  }

  tags = {
    Name        = "${var.workshop_stack_base_name}-users-api"
    Environment = var.environment
    Project     = var.project
  }
}

resource "aws_apigatewayv2_stage" "prod" {
  api_id      = aws_apigatewayv2_api.users.id
  name        = "prod"
  auto_deploy = true

  access_log_settings {
    destination_arn = aws_cloudwatch_log_group.api_gateway.arn
    format = jsonencode({
      requestId               = "$context.requestId"
      extendedRequestId       = "$context.extendedRequestId"
      ip                      = "$context.identity.sourceIp"
      requestTime             = "$context.requestTime"
      httpMethod              = "$context.httpMethod"
      path                    = "$context.path"
      routeKey                = "$context.routeKey"
      status                  = "$context.status"
      protocol                = "$context.protocol"
      responseLength          = "$context.responseLength"
      responseLatency         = "$context.responseLatency"
      integrationLatency      = "$context.integrationLatency"
      integrationStatus       = "$context.integrationStatus"
      integrationErrorMessage = "$context.integrationErrorMessage"
      errorMessage            = "$context.error.message"
      errorResponseType       = "$context.error.responseType"
      authorizerError         = "$context.authorizer.error"
      authorizerLatency       = "$context.authorizer.latency"
      authorizerStatus        = "$context.authorizer.status"
    })
  }

  tags = {
    Name        = "${var.workshop_stack_base_name}-users-api-prod"
    Environment = var.environment
    Project     = var.project
  }
}

resource "aws_cloudwatch_log_group" "api_gateway" {
  name              = "/aws/apigateway/${var.workshop_stack_base_name}-users-api"
  retention_in_days = var.log_retention_days

  tags = {
    Name        = "${var.workshop_stack_base_name}-api-gateway-logs"
    Environment = var.environment
    Project     = var.project
  }
}

resource "aws_apigatewayv2_integration" "users_lambda" {
  api_id                 = aws_apigatewayv2_api.users.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.users.invoke_arn
  integration_method     = "POST"
  payload_format_version = "2.0"
}

# GET /users - List all users
resource "aws_apigatewayv2_route" "get_users" {
  api_id             = aws_apigatewayv2_api.users.id
  route_key          = "GET /users"
  target             = "integrations/${aws_apigatewayv2_integration.users_lambda.id}"
  authorization_type = "CUSTOM"
  authorizer_id      = aws_apigatewayv2_authorizer.cognito.id
}

# GET /users/{userid} - Get a specific user
resource "aws_apigatewayv2_route" "get_user" {
  api_id             = aws_apigatewayv2_api.users.id
  route_key          = "GET /users/{userid}"
  target             = "integrations/${aws_apigatewayv2_integration.users_lambda.id}"
  authorization_type = "CUSTOM"
  authorizer_id      = aws_apigatewayv2_authorizer.cognito.id
}

# PUT /users - Create a new user
resource "aws_apigatewayv2_route" "create_user" {
  api_id             = aws_apigatewayv2_api.users.id
  route_key          = "PUT /users"
  target             = "integrations/${aws_apigatewayv2_integration.users_lambda.id}"
  authorization_type = "CUSTOM"
  authorizer_id      = aws_apigatewayv2_authorizer.cognito.id
}

# PUT /users/{userid} - Update an existing user
resource "aws_apigatewayv2_route" "update_user" {
  api_id             = aws_apigatewayv2_api.users.id
  route_key          = "PUT /users/{userid}"
  target             = "integrations/${aws_apigatewayv2_integration.users_lambda.id}"
  authorization_type = "CUSTOM"
  authorizer_id      = aws_apigatewayv2_authorizer.cognito.id
}

# DELETE /users/{userid} - Delete a user
resource "aws_apigatewayv2_route" "delete_user" {
  api_id             = aws_apigatewayv2_api.users.id
  route_key          = "DELETE /users/{userid}"
  target             = "integrations/${aws_apigatewayv2_integration.users_lambda.id}"
  authorization_type = "CUSTOM"
  authorizer_id      = aws_apigatewayv2_authorizer.cognito.id
}

# OPTIONS /users - CORS preflight
resource "aws_apigatewayv2_route" "options_users" {
  api_id    = aws_apigatewayv2_api.users.id
  route_key = "OPTIONS /users"
  target    = "integrations/${aws_apigatewayv2_integration.users_lambda.id}"
}

# OPTIONS /users/{userid} - CORS preflight
resource "aws_apigatewayv2_route" "options_user" {
  api_id    = aws_apigatewayv2_api.users.id
  route_key = "OPTIONS /users/{userid}"
  target    = "integrations/${aws_apigatewayv2_integration.users_lambda.id}"
}

resource "aws_lambda_permission" "api_gateway" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.users.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.users.execution_arn}/*/*"
}
