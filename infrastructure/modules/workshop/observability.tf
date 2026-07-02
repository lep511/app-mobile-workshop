resource "aws_cloudwatch_dashboard" "main" {
  dashboard_name = "${var.workshop_stack_base_name}-observability"

  dashboard_body = jsonencode({
    widgets = [
      {
        type   = "text"
        x      = 0
        y      = 0
        width  = 24
        height = 1
        properties = {
          markdown = "# ${var.workshop_stack_base_name} — API Gateway"
        }
      },
      {
        type   = "metric"
        x      = 0
        y      = 1
        width  = 8
        height = 6
        properties = {
          metrics = [
            ["AWS/ApiGateway", "Count", "ApiId", aws_apigatewayv2_api.users.id, { stat = "Sum", label = "Total Requests" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "API Requests"
          period = 60
        }
      },
      {
        type   = "metric"
        x      = 8
        y      = 1
        width  = 8
        height = 6
        properties = {
          metrics = [
            ["AWS/ApiGateway", "Latency", "ApiId", aws_apigatewayv2_api.users.id, { stat = "Average", label = "Avg Latency" }],
            ["AWS/ApiGateway", "Latency", "ApiId", aws_apigatewayv2_api.users.id, { stat = "p99", label = "p99 Latency" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "API Latency (ms)"
          period = 60
        }
      },
      {
        type   = "metric"
        x      = 16
        y      = 1
        width  = 8
        height = 6
        properties = {
          metrics = [
            ["AWS/ApiGateway", "4xx", "ApiId", aws_apigatewayv2_api.users.id, { stat = "Sum", label = "4xx Errors" }],
            ["AWS/ApiGateway", "5xx", "ApiId", aws_apigatewayv2_api.users.id, { stat = "Sum", label = "5xx Errors" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "API Errors"
          period = 60
        }
      },
      {
        type   = "text"
        x      = 0
        y      = 7
        width  = 24
        height = 1
        properties = {
          markdown = "# ${var.workshop_stack_base_name} — Users Lambda"
        }
      },
      {
        type   = "metric"
        x      = 0
        y      = 8
        width  = 6
        height = 6
        properties = {
          metrics = [
            ["AWS/Lambda", "Invocations", "FunctionName", aws_lambda_function.users.function_name, { stat = "Sum" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "Users Lambda Invocations"
          period = 60
        }
      },
      {
        type   = "metric"
        x      = 6
        y      = 8
        width  = 6
        height = 6
        properties = {
          metrics = [
            ["AWS/Lambda", "Duration", "FunctionName", aws_lambda_function.users.function_name, { stat = "Average", label = "Avg" }],
            ["AWS/Lambda", "Duration", "FunctionName", aws_lambda_function.users.function_name, { stat = "p99", label = "p99" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "Users Lambda Duration (ms)"
          period = 60
        }
      },
      {
        type   = "metric"
        x      = 12
        y      = 8
        width  = 6
        height = 6
        properties = {
          metrics = [
            ["AWS/Lambda", "Errors", "FunctionName", aws_lambda_function.users.function_name, { stat = "Sum", label = "Errors" }],
            ["AWS/Lambda", "Throttles", "FunctionName", aws_lambda_function.users.function_name, { stat = "Sum", label = "Throttles" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "Users Lambda Errors & Throttles"
          period = 60
        }
      },
      {
        type   = "metric"
        x      = 18
        y      = 8
        width  = 6
        height = 6
        properties = {
          metrics = [
            ["AWS/Lambda", "ConcurrentExecutions", "FunctionName", aws_lambda_function.users.function_name, { stat = "Maximum" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "Users Lambda Concurrency"
          period = 60
        }
      },
      {
        type   = "text"
        x      = 0
        y      = 14
        width  = 24
        height = 1
        properties = {
          markdown = "# ${var.workshop_stack_base_name} — Authorizer Lambda"
        }
      },
      {
        type   = "metric"
        x      = 0
        y      = 15
        width  = 6
        height = 6
        properties = {
          metrics = [
            ["AWS/Lambda", "Invocations", "FunctionName", aws_lambda_function.authorizer.function_name, { stat = "Sum" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "Authorizer Invocations"
          period = 60
        }
      },
      {
        type   = "metric"
        x      = 6
        y      = 15
        width  = 6
        height = 6
        properties = {
          metrics = [
            ["AWS/Lambda", "Duration", "FunctionName", aws_lambda_function.authorizer.function_name, { stat = "Average", label = "Avg" }],
            ["AWS/Lambda", "Duration", "FunctionName", aws_lambda_function.authorizer.function_name, { stat = "p99", label = "p99" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "Authorizer Duration (ms)"
          period = 60
        }
      },
      {
        type   = "metric"
        x      = 12
        y      = 15
        width  = 6
        height = 6
        properties = {
          metrics = [
            ["AWS/Lambda", "Errors", "FunctionName", aws_lambda_function.authorizer.function_name, { stat = "Sum", label = "Errors" }],
            ["AWS/Lambda", "Throttles", "FunctionName", aws_lambda_function.authorizer.function_name, { stat = "Sum", label = "Throttles" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "Authorizer Errors & Throttles"
          period = 60
        }
      },
      {
        type   = "metric"
        x      = 18
        y      = 15
        width  = 6
        height = 6
        properties = {
          metrics = [
            ["AWS/Lambda", "ConcurrentExecutions", "FunctionName", aws_lambda_function.authorizer.function_name, { stat = "Maximum" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "Authorizer Concurrency"
          period = 60
        }
      },
      {
        type   = "text"
        x      = 0
        y      = 21
        width  = 24
        height = 1
        properties = {
          markdown = "# ${var.workshop_stack_base_name} — DynamoDB"
        }
      },
      {
        type   = "metric"
        x      = 0
        y      = 22
        width  = 8
        height = 6
        properties = {
          metrics = [
            ["AWS/DynamoDB", "ConsumedReadCapacityUnits", "TableName", aws_dynamodb_table.users_table.name, { stat = "Sum" }],
            ["AWS/DynamoDB", "ConsumedWriteCapacityUnits", "TableName", aws_dynamodb_table.users_table.name, { stat = "Sum" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "DynamoDB Consumed Capacity"
          period = 60
        }
      },
      {
        type   = "metric"
        x      = 8
        y      = 22
        width  = 8
        height = 6
        properties = {
          metrics = [
            ["AWS/DynamoDB", "SuccessfulRequestLatency", "TableName", aws_dynamodb_table.users_table.name, "Operation", "GetItem", { stat = "Average", label = "GetItem" }],
            ["AWS/DynamoDB", "SuccessfulRequestLatency", "TableName", aws_dynamodb_table.users_table.name, "Operation", "PutItem", { stat = "Average", label = "PutItem" }],
            ["AWS/DynamoDB", "SuccessfulRequestLatency", "TableName", aws_dynamodb_table.users_table.name, "Operation", "Scan", { stat = "Average", label = "Scan" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "DynamoDB Latency (ms)"
          period = 60
        }
      },
      {
        type   = "metric"
        x      = 16
        y      = 22
        width  = 8
        height = 6
        properties = {
          metrics = [
            ["AWS/DynamoDB", "ThrottledRequests", "TableName", aws_dynamodb_table.users_table.name, { stat = "Sum", label = "Throttled Requests" }],
            ["AWS/DynamoDB", "SystemErrors", "TableName", aws_dynamodb_table.users_table.name, { stat = "Sum", label = "System Errors" }]
          ]
          view   = "timeSeries"
          region = data.aws_region.current.region
          title  = "DynamoDB Errors & Throttles"
          period = 60
        }
      }
    ]
  })
}
