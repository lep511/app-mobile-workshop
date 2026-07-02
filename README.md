# Serverless Patterns Workshop

This project implements a serverless REST API on AWS using Rust Lambda functions, Amazon API Gateway HTTP API, Amazon DynamoDB, and Amazon Cognito for authentication. Infrastructure is managed with Terraform and deployed via HCP Terraform Stacks.

## Architecture

```
                       ┌──────────────────┐
                       │  Amazon Cognito  │
                       │  User Pool       │
                       └──────────────────┘
                                 │ JWT validation
                                 ▼
┌──────────┐       ┌──────────────────────────┐       ┌──────────────────┐
│  Client  │▶│  API Gateway (HTTP API)  │──────▶│  Users Lambda    │
│          │       │  + Lambda Authorizer     │       │  (Rust / ARM64)  │
└──────────┘       └──────────────────────────┘       └──────────────────┘
                                                                │
                                                                ▼
                                                      ┌─────────────────┐
                                                      │  DynamoDB       │
                                                      │  (users table)  │
                                                      └─────────────────┘
```

### AWS Services Used

| Service | Purpose |
|---------|---------|
| Amazon API Gateway (HTTP API) | REST endpoint with CORS, access logging, and auto-deploy |
| AWS Lambda | Compute for the Users service and custom Authorizer (Rust, ARM64, `provided.al2023`) |
| Amazon DynamoDB | NoSQL data store for user records (PAY_PER_REQUEST, PITR, encryption at rest) |
| Amazon Cognito | User pool with email-based sign-up, hosted UI, and OAuth 2.0 flows |
| Amazon CloudWatch | Structured JSON logs with 14-day retention, CloudWatch dashboard for API/Lambda/DynamoDB metrics |
| AWS X-Ray | Distributed tracing (active mode) for Lambda functions |

## Project Structure

```
.
├── src/
│   └── functions/
│       ├── authorizer/          # Custom Lambda authorizer (Rust)
│       │   ├── Cargo.toml
│       │   └── src/
│       │       ├── lib.rs       # JWT validation logic
│       │       └── main.rs      # Lambda entry point
│       └── users/               # Users CRUD service (Rust)
│           ├── Cargo.toml
│           └── src/
│               ├── main.rs      # Request router
│               ├── handlers.rs  # Route handlers (CRUD + pagination)
│               ├── models.rs    # User model and request/response types
│               └── errors.rs    # Error types and HTTP response mapping
├── infrastructure/
│   ├── modules/workshop/        # Terraform module
│   │   ├── api-gateway.tf       # HTTP API, routes, integration, stage
│   │   ├── cognito.tf           # User pool, client, domain, groups
│   │   ├── ddb.tf              # DynamoDB table
│   │   ├── lambda-users.tf      # Users Lambda function + IAM
│   │   ├── lambda-authorizer.tf # Authorizer Lambda + IAM + API Gateway authorizer
│   │   ├── observability.tf    # CloudWatch dashboard (API, Lambda, DynamoDB)
│   │   ├── main.tf             # Provider requirements
│   │   ├── variables.tf        # Module input variables
│   │   └── outputs.tf          # Module outputs
│   ├── components.tfcomponent.hcl  # HCP Terraform Stacks component
│   ├── deployments.tfdeploy.hcl    # HCP Terraform Stacks deployments (dev/prod)
│   ├── variables.tfcomponent.hcl   # Stack-level variables
│   ├── outputs.tfcomponent.hcl     # Stack-level outputs
│   └── dist/                       # Pre-built Lambda deployment packages
├── scripts/
│   ├── build-users-lambda.sh       # Build and package users Lambda
│   └── build-authorizer-lambda.sh  # Build and package authorizer Lambda
├── tests/
│   └── integration/                # Integration tests (bash, invokes deployed Lambda)
└── .github/workflows/
    ├── ci.yml                      # CI: unit tests, formatting, clippy, Terraform validate
    ├── deploy-lambdas.yml          # CD: unit tests → dev deploy → integration tests → prod deploy
    └── deploy-lambdas-env.yml      # Reusable workflow for per-environment Lambda deployment
```

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [cargo-lambda](https://www.cargo-lambda.info/guide/installation.html) for building and deploying Lambda functions
- [Terraform](https://developer.hashicorp.com/terraform/install) >= 1.0.0
- [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html) configured with appropriate credentials
- (Optional) Bash for integration tests

## Getting Started

### 1. Clone the Repository

```bash
git clone <repository-url>
cd app-mobile
```

### 2. Build Lambda Functions

Build both Lambda functions for the ARM64 architecture:

```bash
# Build the users Lambda
./scripts/build-users-lambda.sh

# Build the authorizer Lambda
./scripts/build-authorizer-lambda.sh
```

This compiles the Rust code with `cargo-lambda`, targets `aarch64-unknown-linux-gnu`, and packages the bootstrap binary into ZIP files under `infrastructure/dist/`.

### 3. Deploy Infrastructure

```bash
cd infrastructure
terraform init
terraform plan
terraform apply
```

To deploy to a specific region:

```bash
terraform apply -var="region=us-east-1"
```

See [infrastructure/HCP_TERRAFORM.md](infrastructure/HCP_TERRAFORM.md) for HCP Terraform Stacks deployment instructions.

## API Reference

All routes (except OPTIONS) require a valid Cognito access token in the `Authorization` header.

**Base URL:** `{api_gateway_stage_url}/`

### Authentication

```bash
# Obtain an access token from Cognito
TOKEN=$(aws cognito-idp initiate-auth \
  --client-id <client-id> \
  --auth-flow USER_PASSWORD_AUTH \
  --auth-parameters USERNAME=<email>,PASSWORD=<password> \
  --query 'AuthenticationResult.AccessToken' \
  --output text)
```

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `PUT` | `/users` | Create a new user |
| `GET` | `/users` | List users (paginated) |
| `GET` | `/users/{userid}` | Get a user by ID |
| `PUT` | `/users/{userid}` | Update an existing user |
| `DELETE` | `/users/{userid}` | Delete a user |
| `OPTIONS` | `/users`, `/users/{userid}` | CORS preflight (no auth required) |

### Request/Response Examples

#### Create User

```bash
curl -X PUT "${API_URL}/users" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "John Doe",
    "email": "john@example.com",
    "phone": "+1-555-0100"
  }'
```

**Response (201):**
```json
{
  "userid": "550e8400-e29b-41d4-a716-446655440000",
  "name": "John Doe",
  "email": "john@example.com",
  "phone": "+1-555-0100"
}
```

#### List Users

```bash
curl "${API_URL}/users?limit=10" \
  -H "Authorization: Bearer ${TOKEN}"
```

**Response (200):**
```json
{
  "users": [
    {
      "userid": "550e8400-e29b-41d4-a716-446655440000",
      "name": "John Doe",
      "email": "john@example.com",
      "phone": "+1-555-0100"
    }
  ],
  "next_token": "eyJ1c2VyaWQiOiAiNTUwZTg0MDAuLi4ifQ"
}
```

Query parameters:
- `limit` (optional): Number of results per page (1-100, default: 20)
- `next_token` (optional): Pagination token from previous response

#### Get User

```bash
curl "${API_URL}/users/{userid}" \
  -H "Authorization: Bearer ${TOKEN}"
```

#### Update User

```bash
curl -X PUT "${API_URL}/users/{userid}" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Jane Doe",
    "phone": "+1-555-0200"
  }'
```

At least one field (`name`, `email`, or `phone`) must be provided.

#### Delete User

```bash
curl -X DELETE "${API_URL}/users/{userid}" \
  -H "Authorization: Bearer ${TOKEN}"
```

**Response:** `204 No Content`

### Error Responses

All errors return a JSON body with a `message` field:

```json
{
  "message": "User 550e8400-... not found"
}
```

| Status Code | Description |
|-------------|-------------|
| 400 | Validation error (missing/invalid fields) |
| 404 | Resource not found |
| 405 | Method not allowed |
| 409 | Conflict (duplicate resource) |
| 500 | Internal server error |

## Testing

### Unit Tests

```bash
# Users Lambda
cd src/functions/users && cargo test

# Authorizer Lambda
cd src/functions/authorizer && cargo test
```

### Integration Tests

Integration tests invoke the deployed Lambda function directly via the AWS CLI:

```bash
# Set the function name and region (defaults: workshop-dev-users, us-west-2)
export FUNCTION_NAME="workshop-dev-users"
export AWS_REGION="us-west-2"

./tests/integration/test_users_lambda.sh
```

The integration test suite covers the full CRUD lifecycle:
1. Create a user
2. Get the created user
3. List users with pagination
4. Update the user
5. Delete the user
6. Verify deletion (404)
7. Validation error handling

## CI/CD

### CI Pipeline (`ci.yml`)

Runs on every pull request and push to `main`:

1. **Test Users Lambda** — `cargo test`, `cargo fmt --check`, `cargo clippy`
2. **Test Authorizer Lambda** — same checks
3. **Validate Infrastructure** — `terraform fmt -check`, `terraform validate`

### CD Pipeline (`deploy-lambdas.yml`)

Triggered on pushes to `main` that modify `src/functions/**`, or via manual `workflow_dispatch`:

1. **Unit Tests** — runs tests, fmt, and clippy for both Lambda functions in parallel
2. **Deploy to Dev** — builds and deploys both Lambdas to the dev environment
3. **Integration Tests** — runs the full CRUD integration test suite against dev
4. **Deploy to Prod** — promotes to production only after integration tests pass

The pipeline uses a reusable workflow (`deploy-lambdas-env.yml`) for per-environment deployments with `cargo lambda build --release --arm64` and `cargo lambda deploy`.

### Required Secrets/Variables (per environment)

| Name | Type | Description |
|------|------|-------------|
| `AWS_ACCESS_KEY_ID` | Secret | AWS access key for deployment |
| `AWS_SECRET_ACCESS_KEY` | Secret | AWS secret key for deployment |
| `AWS_SESSION_TOKEN` | Secret | (Optional) AWS session token |
| `TEST_USERNAME` | Secret | Test user for integration tests (dev only) |
| `TEST_PASSWORD` | Secret | Test user password for integration tests (dev only) |
| `USERS_LAMBDA_FUNCTION_NAME` | Variable | Name of the Users Lambda function |
| `AUTHORIZER_LAMBDA_FUNCTION_NAME` | Variable | Name of the Authorizer Lambda function |
| `AWS_DEFAULT_REGION` | Variable | AWS region for deployment |
| `API_GATEWAY_STAGE_URL` | Variable | API base URL for integration tests (dev only) |
| `COGNITO_CLIENT_ID` | Variable | Cognito client ID for integration tests (dev only) |

## Infrastructure Details

### Terraform Module Inputs

| Variable | Description | Default |
|----------|-------------|---------|
| `workshop_stack_base_name` | Base name prefix for all resources | `"workshop"` |
| `environment` | Environment name (dev/prod) | — |
| `project` | Project name tag | `"Serverless Patterns"` |
| `region` | AWS region | `"us-west-2"` |
| `cors_allowed_origins` | Allowed CORS origins | `["*"]` |
| `lambda_log_level` | Log level for Lambda functions (RUST_LOG format) | `"info"` |
| `log_retention_days` | Number of days to retain CloudWatch logs | `14` |

### Terraform Outputs

| Output | Description |
|--------|-------------|
| `api_gateway_stage_url` | Full invoke URL for the prod stage |
| `api_gateway_endpoint` | HTTP API Gateway endpoint |
| `cognito_user_pool_id` | Cognito User Pool ID |
| `cognito_user_pool_client_id` | Cognito User Pool Client ID |
| `cognito_login_url` | Cognito hosted UI login URL |
| `users_table_name` | DynamoDB table name |
| `users_lambda_function_name` | Users Lambda function name |
| `authorizer_lambda_function_arn` | Authorizer Lambda ARN |
| `cloudwatch_dashboard_name` | CloudWatch observability dashboard name |
| `cloudwatch_dashboard_url` | Direct URL to the CloudWatch dashboard |

### HCP Terraform Stacks

This project uses [HCP Terraform Stacks](https://developer.hashicorp.com/terraform/language/stacks) with two deployments:

- **dev** — auto-approved, prefix `workshop-dev`
- **prod** — requires manual approval, prefix `workshop-prod`

Authentication uses OIDC with workload identity tokens (no static credentials).

## Observability

The infrastructure includes a CloudWatch dashboard (`{base_name}-observability`) with panels for:

- **API Gateway** — request count, average/p99 latency, 4xx/5xx error rates
- **Users Lambda** — invocations, duration (avg/p99), errors & throttles, concurrency
- **Authorizer Lambda** — invocations, duration (avg/p99), errors & throttles, concurrency
- **DynamoDB** — consumed read/write capacity, operation latency (GetItem, PutItem, Scan), throttles & system errors

Both Lambda functions emit JSON-structured logs with request ID correlation and configurable log level via the `lambda_log_level` Terraform variable. AWS X-Ray active tracing is enabled on all Lambda functions for distributed request tracing.

## Security

- **Authentication:** All API routes are protected by a custom Lambda authorizer that validates Cognito JWT access tokens (RS256, issuer/audience/expiry checks)
- **Authorization result caching:** Authorizer responses are cached for 300 seconds to reduce latency
- **Least-privilege IAM:** Each Lambda has a dedicated role with only the permissions it needs
- **Encryption:** DynamoDB server-side encryption enabled; data in transit via HTTPS
- **Point-in-time recovery:** Enabled on the DynamoDB table
- **Deletion protection:** DynamoDB table has deletion protection enabled
- **Logging:** JSON-structured logs with request ID correlation for API Gateway and Lambda (configurable retention, default 14 days)
- **Tracing:** AWS X-Ray active tracing on all Lambda functions
- **Dashboard:** CloudWatch observability dashboard covering API, Lambda, and DynamoDB metrics
- **CORS:** Configurable allowed origins with explicit headers and methods
- **Password policy:** Minimum 8 characters with uppercase, lowercase, numbers, and symbols required

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
