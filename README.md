# Serverless Patterns Workshop

This project implements a serverless REST API on AWS using Rust Lambda functions, Amazon API Gateway HTTP API, Amazon DynamoDB, and Amazon Cognito for authentication. Infrastructure is managed with Terraform and deployed via HCP Terraform Stacks.

## Architecture

```
                          ┌──────────────────┐
                          │   Amazon Cognito  │
                          │    User Pool      │
                          └────────┬─────────┘
                                   │ JWT validation
                                   ▼
┌──────────┐       ┌───────────────────────────────┐       ┌────────────────┐
│  Client  │──────▶│  API Gateway (HTTP API)       │──────▶│  Users Lambda  │
└──────────┘       │  + Lambda Authorizer          │       │  (Rust/ARM64)  │
                   └───────────────────────────────┘       └───────┬────────┘
                                                                   │
                                                                   ▼
                                                          ┌────────────────┐
                                                          │   DynamoDB     │
                                                          │  (users table) │
                                                          └────────────────┘
```

### AWS Services Used

| Service | Purpose |
|---------|---------|
| Amazon API Gateway (HTTP API) | REST endpoint with CORS, access logging, and auto-deploy |
| AWS Lambda | Compute for the Users service and custom Authorizer (Rust, ARM64, `provided.al2023`) |
| Amazon DynamoDB | NoSQL data store for user records (PAY_PER_REQUEST, PITR, encryption at rest) |
| Amazon Cognito | User pool with email-based sign-up, hosted UI, and OAuth 2.0 flows |
| Amazon CloudWatch | Structured logs with 14-day retention for API Gateway and Lambda |
| AWS X-Ray | Distributed tracing for Lambda functions |

## Project Structure

```
.
├── src/
│   └── functions/
│       ├── authorizer/          # Custom Lambda authorizer (Rust)
│       │   ├── Cargo.toml
│       │   └── src/
│       │       └── main.rs      # JWT validation against Cognito JWKS
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
│   ├── unit/users/                 # Unit tests (pytest)
│   └── integration/                # Integration tests (bash, invokes deployed Lambda)
└── .github/workflows/
    └── deploy-users-lambda.yml     # CI/CD pipeline for users Lambda
```

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [cargo-lambda](https://www.cargo-lambda.info/guide/installation.html) for building and deploying Lambda functions
- [Terraform](https://developer.hashicorp.com/terraform/install) >= 1.0.0
- [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html) configured with appropriate credentials
- Python 3.x and pytest (for unit tests)

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
pytest tests/unit/
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

The GitHub Actions workflow (`.github/workflows/deploy-users-lambda.yml`) automatically deploys the Users Lambda when changes are pushed to `main` under `src/functions/users/`:

1. Checks out the repository
2. Installs the Rust stable toolchain with `aarch64-unknown-linux-gnu` target
3. Caches the Cargo registry and build artifacts
4. Builds with `cargo lambda build --release --arm64`
5. Deploys with `cargo lambda deploy`

### Required Secrets/Variables

| Name | Type | Description |
|------|------|-------------|
| `LAMBDA_DEPLOYER_ACCESS_KEY_ID` | Secret | AWS access key for deployment |
| `LAMBDA_DEPLOYER_SECRET_ACCESS_KEY` | Secret | AWS secret key for deployment |
| `USERS_LAMBDA_FUNCTION_NAME` | Variable | Name of the target Lambda function |
| `AWS_REGION` | Variable | AWS region for deployment |

## Infrastructure Details

### Terraform Module Inputs

| Variable | Description | Default |
|----------|-------------|---------|
| `workshop_stack_base_name` | Base name prefix for all resources | `"workshop"` |
| `environment` | Environment name (dev/prod) | — |
| `project` | Project name tag | `"Serverless Patterns"` |
| `region` | AWS region | `"us-west-2"` |
| `cors_allowed_origins` | Allowed CORS origins | `["*"]` |

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

### HCP Terraform Stacks

This project uses [HCP Terraform Stacks](https://developer.hashicorp.com/terraform/language/stacks) with two deployments:

- **dev** — auto-approved, prefix `workshop-dev`
- **prod** — requires manual approval, prefix `workshop-prod`

Authentication uses OIDC with workload identity tokens (no static credentials).

## Security

- **Authentication:** All API routes are protected by a custom Lambda authorizer that validates Cognito JWT access tokens (RS256, issuer/audience/expiry checks)
- **Authorization result caching:** Authorizer responses are cached for 300 seconds to reduce latency
- **Least-privilege IAM:** Each Lambda has a dedicated role with only the permissions it needs
- **Encryption:** DynamoDB server-side encryption enabled; data in transit via HTTPS
- **Point-in-time recovery:** Enabled on the DynamoDB table
- **Deletion protection:** DynamoDB table has deletion protection enabled
- **Logging:** Structured JSON logs for API Gateway and Lambda with 14-day retention
- **Tracing:** AWS X-Ray active tracing on all Lambda functions
- **CORS:** Configurable allowed origins with explicit headers and methods
- **Password policy:** Minimum 8 characters with uppercase, lowercase, numbers, and symbols required

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
