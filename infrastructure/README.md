# Serverless Patterns - Infrastructure (Terraform Stacks)

Terraform Stacks configuration for deploying the Serverless Patterns workshop infrastructure across multiple environments on AWS.

## Prerequisites

- [Terraform](https://www.terraform.io/downloads) >= 1.13.0
- HCP Terraform account with Stacks enabled (Settings > General)
- An AWS account with an OIDC trust relationship configured for HCP Terraform

## Project Structure

```
infrastructure/
├── components.tfcomponent.hcl    # Defines the workshop component and providers
├── variables.tfcomponent.hcl     # Stack-level input variables
├── outputs.tfcomponent.hcl       # Stack outputs per deployment
├── deployments.tfdeploy.hcl      # Environment deployments (dev, prod)
├── .terraform-version            # Required Terraform version for HCP Terraform
├── .terraform.lock.hcl           # Provider lock file
└── modules/
    └── workshop/                 # Workshop infrastructure module
        ├── main.tf               # Provider requirements
        ├── ddb.tf                # DynamoDB table
        ├── lambda-users.tf       # Users Lambda function and IAM execution role
        ├── api-gateway.tf        # HTTP API Gateway with routes and authorizer integration
        ├── cognito.tf            # Cognito User Pool, client, domain, and admin group
        ├── lambda-authorizer.tf  # Lambda authorizer for API Gateway (Cognito JWT validation)
        ├── placeholder.zip       # Placeholder binary for initial Lambda creation
        ├── variables.tf          # Module variables
        └── outputs.tf            # Module outputs
```

## Deployments

| Deployment | Base Name | Approval | Description |
|------------|-----------|----------|-------------|
| `dev` | `workshop-dev` | Auto-approve | Development environment |
| `prod` | `workshop-prod` | Manual approval required | Production environment |

## Authentication

The Stack uses OIDC (OpenID Connect) via `identity_token` blocks to authenticate with AWS. Each deployment assumes the IAM role `hcp-terraform-stacks-role` using a JWT issued by HCP Terraform — no static credentials needed.

The provider is configured with `assume_role_with_web_identity` in `components.tfcomponent.hcl`.

## Resources

Each deployment creates:

| Resource | Naming Pattern | Example (dev) |
|----------|---------------|---------------|
| DynamoDB table | `{base_name}-users` | `workshop-dev-users` |
| Lambda function (Rust, arm64) | `{base_name}-users` | `workshop-dev-users` |
| HTTP API Gateway (v2) | `{base_name}-users-api` | `workshop-dev-users-api` |
| IAM execution role | `{base_name}-users-lambda` | `workshop-dev-users-lambda` |
| Lambda authorizer (Rust, arm64) | `{base_name}-authorizer` | `workshop-dev-authorizer` |
| IAM execution role (authorizer) | `{base_name}-authorizer-lambda` | `workshop-dev-authorizer-lambda` |
| API Gateway authorizer | `{base_name}-cognito-authorizer` | `workshop-dev-cognito-authorizer` |
| Cognito User Pool | `{base_name}_UserPool` | `workshop-dev_UserPool` |
| Cognito User Pool Client | `{base_name}-client` | `workshop-dev-client` |
| Cognito Domain | `{base_name}` | `workshop-dev` |
| Cognito User Group | `Administrators` | `Administrators` |
| CloudWatch log groups | `/aws/lambda/...`, `/aws/apigateway/...` | 14-day retention |

## Local Commands

```bash
# Initialize the Stack (downloads providers)
terraform stacks init

# Format configuration
terraform stacks fmt

# Validate configuration
terraform stacks validate

# Lock providers — updates .terraform.lock.hcl with hashes for all
# required_providers. Must be re-run whenever a provider is added or
# its version constraint changes. The lockfile is committed to VCS so
# that HCP Terraform uses the exact same provider binaries.
terraform stacks providers-lock

# Upload configuration to HCP Terraform — packages all .hcl files,
# the lockfile, and the modules/ directory, then pushes them as a new
# configuration version for the named stack. HCP Terraform immediately
# queues a plan for every deployment (dev, prod) defined in
# deployments.tfdeploy.hcl.
terraform stacks configuration upload \
  -organization-name "lep511" \
  -project-name "workshop-app-mobile" \
  -stack-name "stack-mobile-app-v2"
```

### Typical workflow

```bash
# 1. Make changes to .hcl or module .tf files

# 2. Format and validate locally
terraform stacks fmt
terraform stacks validate

# 3. If you added/changed a provider, regenerate the lockfile
terraform stacks providers-lock

# 4. Upload to HCP Terraform (triggers plans for all deployments)
terraform stacks configuration upload \
  -organization-name "lep511" \
  -project-name "workshop-app-mobile" \
  -stack-name "stack-mobile-app-v2"

# 5. dev auto-applies; prod requires manual approval in the UI
```

## Deploying via HCP Terraform

For complete HCP Terraform deployment details (org, stack, IAM policy, troubleshooting), see [HCP_TERRAFORM.md](HCP_TERRAFORM.md).

Stacks are deployed through HCP Terraform (not CLI apply):

1. Upload or push configuration to your connected VCS repository
2. HCP Terraform automatically plans changes for all deployments
3. `dev` is auto-approved and applied immediately
4. `prod` requires manual approval in the HCP Terraform UI

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `region` | `us-west-2` | AWS region for resource deployment |
| `role_arn` | *(required)* | IAM role ARN for OIDC authentication |
| `identity_token` | *(required)* | JWT identity token (ephemeral, set per deployment) |
| `workshop_stack_base_name` | `workshop` | Base name for the workshop stack |
| `environment` | *(required)* | Environment name (set per deployment) |
| `project` | `Serverless Patterns` | Project name |
| `cors_allowed_origins` | `["*"]` | Allowed origins for API Gateway CORS |

## Lambda Deployment (Cargo Lambda)

The Lambda function is written in Rust using the `lambda_http` crate (compatible with API Gateway HTTP API payload format 2.0). Terraform manages the full lifecycle including code updates — when `users_lambda_zip_path` changes, Terraform deploys the new binary.

### Build and deploy via Terraform

```bash
# Build the Rust Lambda for arm64
cd src/functions/users
cargo lambda build --release --arm64

# Package the binary
cd target/lambda/users-lambda
zip /path/to/dist/users-lambda.zip bootstrap

# Deploy via Terraform (pass the zip path)
cd infrastructure
terraform stacks configuration upload \
  -organization-name "lep511" \
  -project-name "workshop-app-mobile" \
  -stack-name "stack-mobile-app-v2"
```

### Manual code update (without Terraform)

```bash
# Build and deploy directly to an existing function
cd src/functions/users
cargo lambda build --release --arm64
cd target/lambda/users-lambda
zip /tmp/users-lambda.zip bootstrap
aws lambda update-function-code \
  --function-name workshop-dev-users \
  --zip-file fileb:///tmp/users-lambda.zip \
  --region us-west-2
```

### Initial deployment

On first `terraform apply`, the Lambda is created with a `placeholder.zip` (included in the module). After creation, update the function code using one of the methods above.

## API Endpoints

The HTTP API Gateway (v2) exposes the following routes on the `prod` stage:

| Method | Route | Auth | Description |
|--------|-------|------|-------------|
| GET | `/users` | Yes | List users (supports `?limit=N&next_token=TOKEN`) |
| GET | `/users/{userid}` | Yes | Get a specific user |
| PUT | `/users` | Yes | Create a new user (auto-generated UUID) |
| PUT | `/users/{userid}` | Yes | Update an existing user (partial update) |
| DELETE | `/users/{userid}` | Yes | Delete a user |
| OPTIONS | `/users` | No | CORS preflight |
| OPTIONS | `/users/{userid}` | No | CORS preflight |

All authenticated routes require a valid Cognito access token in the `Authorization: Bearer <token>` header. The Lambda authorizer validates the JWT signature, issuer, client_id, and token_use claims with a 300-second cache TTL.

### curl Examples

Replace `$API_URL` with your stage URL (e.g., `https://abc123.execute-api.us-west-2.amazonaws.com/prod`).

```bash
# Set your API endpoint
export API_URL="https://<api-id>.execute-api.us-west-2.amazonaws.com/prod"

# Get a Cognito access token (replace with your credentials)
export TOKEN="<your-cognito-access-token>"

# List all users (with pagination)
curl -s "$API_URL/users?limit=10" \
  -H "Authorization: Bearer $TOKEN" | jq .

# Get a specific user
curl -s "$API_URL/users/550e8400-e29b-41d4-a716-446655440001" \
  -H "Authorization: Bearer $TOKEN" | jq .

# Create a new user
curl -s -X PUT "$API_URL/users" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Jane Doe","email":"jane@example.com","phone":"+1-555-0200"}' | jq .

# Update an existing user
curl -s -X PUT "$API_URL/users/550e8400-e29b-41d4-a716-446655440001" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Alice J. Updated","phone":"+1-555-9999"}' | jq .

# Delete a user
curl -s -X DELETE "$API_URL/users/550e8400-e29b-41d4-a716-446655440005" \
  -H "Authorization: Bearer $TOKEN" -w "\nHTTP Status: %{http_code}\n"

# CORS preflight check
curl -s -X OPTIONS "$API_URL/users" \
  -H "Origin: http://localhost:3000" \
  -H "Access-Control-Request-Method: PUT" \
  -H "Access-Control-Request-Headers: content-type" -v 2>&1 | grep -i "access-control"
```

### Outputs

After deployment, the following outputs are available:

| Output | Description |
|--------|-------------|
| `api_gateway_endpoint` | Base API endpoint URL |
| `api_gateway_execution_arn` | Execution ARN (used for Lambda permissions) |
| `api_gateway_stage_url` | Full invoke URL for the prod stage |
| `cognito_user_pool_id` | Cognito User Pool ID |
| `cognito_user_pool_client_id` | Cognito User Pool Client ID |
| `cognito_domain_url` | Cognito hosted UI domain URL |
| `cognito_login_url` | Cognito login URL for the hosted UI |
| `authorizer_lambda_function_arn` | ARN of the authorizer Lambda function |
| `authorizer_id` | ID of the API Gateway authorizer |

## Testing

Run the integration test suite against the deployed Lambda:

```bash
# Uses default: FUNCTION_NAME=workshop-dev-users, AWS_REGION=us-west-2
./tests/integration/test_users_lambda.sh

# Override for a different environment
FUNCTION_NAME=workshop-prod-users ./tests/integration/test_users_lambda.sh
```

The test script validates all CRUD operations, pagination, and error handling (7 tests).
