resource "aws_iam_role" "authorizer_lambda" {
  name = "${var.workshop_stack_base_name}-authorizer-lambda"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
        Action = "sts:AssumeRole"
      }
    ]
  })

  tags = {
    Name        = "${var.workshop_stack_base_name}-authorizer-lambda"
    Environment = var.environment
    Project     = var.project
  }
}

resource "aws_iam_role_policy_attachment" "authorizer_lambda_basic_execution" {
  role       = aws_iam_role.authorizer_lambda.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "authorizer_lambda_cognito" {
  name = "cognito-access"
  role = aws_iam_role.authorizer_lambda.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "cognito-idp:GetUser",
          "cognito-idp:ListUsers",
          "cognito-idp:AdminGetUser",
          "cognito-idp:AdminListGroupsForUser"
        ]
        Resource = aws_cognito_user_pool.main.arn
      }
    ]
  })
}

locals {
  authorizer_lambda_zip_path = "${path.module}/../../dist/authorizer-lambda.zip"
}

resource "aws_lambda_function" "authorizer" {
  function_name = "${var.workshop_stack_base_name}-authorizer"
  role          = aws_iam_role.authorizer_lambda.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["arm64"]
  timeout       = 10
  memory_size   = 256

  filename         = local.authorizer_lambda_zip_path
  source_code_hash = filebase64sha256(local.authorizer_lambda_zip_path)

  environment {
    variables = {
      USER_POOL_ID     = aws_cognito_user_pool.main.id
      CLIENT_ID        = aws_cognito_user_pool_client.main.id
      ADMIN_GROUP_NAME = aws_cognito_user_group.administrators.name
      RUST_LOG         = var.lambda_log_level
    }
  }

  tracing_config {
    mode = "Active"
  }

  tags = {
    Name        = "${var.workshop_stack_base_name}-authorizer"
    Environment = var.environment
    Project     = var.project
  }
}

resource "aws_cloudwatch_log_group" "authorizer_lambda" {
  name              = "/aws/lambda/${aws_lambda_function.authorizer.function_name}"
  retention_in_days = 14

  tags = {
    Name        = "${var.workshop_stack_base_name}-authorizer-logs"
    Environment = var.environment
    Project     = var.project
  }
}

resource "aws_apigatewayv2_authorizer" "cognito" {
  api_id                            = aws_apigatewayv2_api.users.id
  authorizer_type                   = "REQUEST"
  authorizer_uri                    = aws_lambda_function.authorizer.invoke_arn
  authorizer_payload_format_version = "2.0"
  name                              = "${var.workshop_stack_base_name}-cognito-authorizer"
  identity_sources                  = ["$request.header.Authorization"]
  authorizer_result_ttl_in_seconds  = 300
  enable_simple_responses           = true
}

resource "aws_lambda_permission" "authorizer_api_gateway" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.authorizer.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.users.execution_arn}/*/*"
}
