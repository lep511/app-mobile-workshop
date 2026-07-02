data "aws_region" "current" {}
data "aws_caller_identity" "current" {}

locals {
  users_lambda_zip_path = "${path.module}/../../dist/users-lambda.zip"
}

resource "aws_iam_role" "users_lambda" {
  name = "${var.workshop_stack_base_name}-users-lambda"

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
    Name        = "${var.workshop_stack_base_name}-users-lambda"
    Environment = var.environment
    Project     = var.project
  }
}

resource "aws_iam_role_policy" "users_lambda_dynamodb" {
  name = "dynamodb-access"
  role = aws_iam_role.users_lambda.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "dynamodb:GetItem",
          "dynamodb:PutItem",
          "dynamodb:UpdateItem",
          "dynamodb:DeleteItem",
          "dynamodb:Scan"
        ]
        Resource = aws_dynamodb_table.users_table.arn
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "users_lambda_basic_execution" {
  role       = aws_iam_role.users_lambda.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy_attachment" "users_lambda_xray" {
  role       = aws_iam_role.users_lambda.name
  policy_arn = "arn:aws:iam::aws:policy/AWSXRayDaemonWriteAccess"
}

resource "aws_lambda_function" "users" {
  function_name = "${var.workshop_stack_base_name}-users"
  role          = aws_iam_role.users_lambda.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["arm64"]
  timeout       = 10
  memory_size   = 256

  filename         = local.users_lambda_zip_path
  source_code_hash = filebase64sha256(local.users_lambda_zip_path)

  environment {
    variables = {
      USERS_TABLE_NAME = aws_dynamodb_table.users_table.name
      RUST_LOG         = var.lambda_log_level
    }
  }

  tracing_config {
    mode = "Active"
  }


  tags = {
    Name        = "${var.workshop_stack_base_name}-users"
    Environment = var.environment
    Project     = var.project
  }
}


resource "aws_cloudwatch_log_group" "users_lambda" {
  name              = "/aws/lambda/${aws_lambda_function.users.function_name}"
  retention_in_days = 14

  tags = {
    Name        = "${var.workshop_stack_base_name}-users-logs"
    Environment = var.environment
    Project     = var.project
  }
}
