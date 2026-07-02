# Deploying to HCP Terraform with Stacks

## Stack Information

| Field | Value |
|-------|-------|
| Organization | `lep511` |
| Project | `workshop-app-mobile` |
| Stack Name | `stack-mobile-app-v2` |
| Stack ID | `st-kM5CMNrT3yW65Bvi` |
| HCP Terraform URL | https://app.terraform.io/app/lep511/projects/prj-Ybw6DA7aRAC8LSAq/stacks/st-kM5CMNrT3yW65Bvi |

## Prerequisites

1. **Terraform CLI** >= 1.15.0 with Stacks plugin installed
2. **HCP Terraform account** with Stacks enabled (Settings > General)
3. **HCP Terraform token** configured locally:
   ```bash
   terraform login
   ```
4. **AWS IAM role** with OIDC trust for HCP Terraform:
   - Role ARN: `arn:aws:iam::375920412105:role/hcp-terraform-stacks-role`
   - Trust policy must allow `app.terraform.io` as an OIDC provider

## AWS Authentication

This stack uses **OIDC (OpenID Connect)** — no static AWS credentials are stored in HCP Terraform.

Each deployment assumes the IAM role using a JWT token issued by HCP Terraform:

```hcl
provider "aws" "this" {
  config {
    region = var.region
    assume_role_with_web_identity {
      role_arn           = var.role_arn
      web_identity_token = var.identity_token
    }
  }
}
```

The `identity_token` is ephemeral and generated per plan/apply.

## Deployments

| Deployment | Base Name | Region | Approval |
|------------|-----------|--------|----------|
| `dev` | `workshop-dev` | us-west-2 | Auto-approve |
| `prod` | `workshop-prod` | us-west-2 | Manual approval required |

## Deploy Commands

All commands must be run from the `infrastructure/` directory.

### First-time setup

```bash
cd infrastructure

# Initialize (downloads providers, validates config)
terraform stacks init

# Lock provider versions
terraform stacks providers-lock
```

### Deploy workflow

```bash
cd infrastructure

# 1. Validate configuration locally
terraform stacks validate

# 2. Format files
terraform stacks fmt

# 3. If providers changed, update the lockfile
terraform stacks providers-lock

# 4. Upload configuration to HCP Terraform
terraform stacks configuration upload \
  -organization-name "lep511" \
  -project-name "workshop-app-mobile" \
  -stack-name "stack-mobile-app-v2"
```

After upload:
- HCP Terraform plans all deployments automatically
- `dev` auto-approves and applies if the plan succeeds
- `prod` requires manual approval in the HCP Terraform UI

### Quick deploy (one-liner)

```bash
cd infrastructure && terraform stacks fmt && terraform stacks validate && terraform stacks configuration upload -organization-name "lep511" -project-name "workshop-app-mobile" -stack-name "stack-mobile-app-v2"
```

## Configuration Variables

| Variable | Default | Per-Deployment | Description |
|----------|---------|----------------|-------------|
| `region` | `us-west-2` | No | AWS region |
| `role_arn` | *(required)* | Yes | IAM role ARN for OIDC |
| `identity_token` | *(ephemeral)* | Yes | JWT from HCP Terraform |
| `workshop_stack_base_name` | `workshop` | Yes | Resource naming prefix |
| `environment` | *(required)* | Yes | `dev` or `prod` |
| `project` | `Serverless Patterns` | No | Project tag |
| `cors_allowed_origins` | `["*"]` | No | API Gateway CORS origins |

## Outputs

After a successful apply, each deployment exposes:

| Output | Description |
|--------|-------------|
| `users_table_arn` | DynamoDB table ARN |
| `users_table_id` | DynamoDB table ID |
| `users_table_name` | DynamoDB table name |
| `api_gateway_endpoint` | HTTP API Gateway endpoint URL |
| `api_gateway_execution_arn` | Execution ARN of the HTTP API Gateway |
| `api_gateway_stage_url` | Full stage invoke URL for the Prod stage |
| `cognito_user_pool_id` | Cognito User Pool ID |
| `cognito_user_pool_client_id` | Cognito User Pool Client ID |
| `cognito_domain_url` | Cognito hosted UI domain URL |
| `cognito_login_url` | Cognito login URL for the hosted UI |
| `authorizer_lambda_function_arn` | ARN of the authorizer Lambda function |
| `authorizer_id` | ID of the API Gateway authorizer |

## Auto-Approve Rules

```hcl
deployment_auto_approve "dev_only" {
  check {
    condition = context.plan.deployment.deployment_name == "dev"
    reason    = "Only dev is auto-approved."
  }
}
```

Plans for the `dev` deployment are automatically approved. All other deployments (`prod`) require manual approval via the HCP Terraform UI.

## IAM Role Policy

The `hcp-terraform-stacks-role` requires the following permissions:

| Service | Actions | Resource Scope |
|---------|---------|----------------|
| DynamoDB | CreateTable, DeleteTable, DescribeTable, UpdateTable, etc. | `arn:aws:dynamodb:us-west-2:375920412105:table/workshop-*` |
| IAM | CreateRole, DeleteRole, AttachRolePolicy, PassRole, etc. | `arn:aws:iam::375920412105:role/workshop-*` |
| Lambda | CreateFunction, UpdateFunctionCode, AddPermission, etc. | `arn:aws:lambda:us-west-2:375920412105:function:workshop-*` |
| API Gateway | GET, POST, PUT, PATCH, DELETE | `arn:aws:apigateway:us-west-2::/*` |
| CloudWatch Logs | CreateLogGroup, DescribeLogGroups, PutRetentionPolicy, etc. | `*` (DescribeLogGroups) + scoped for mutations |
| CloudWatch Logs | CreateLogDelivery, DeleteLogDelivery, etc. | `*` (required for API Gateway access logging) |
| Cognito | CreateUserPool, DeleteUserPool, UpdateUserPool, CreateUserPoolClient, CreateUserPoolDomain, CreateGroup, etc. | `*` |

## Troubleshooting

### "Missing .terraform-version file"

You're running from the wrong directory. Run from `infrastructure/`:
```bash
cd infrastructure && terraform stacks configuration upload ...
```

### "Provider missing from lockfile"

A provider was added or changed. Regenerate the lockfile:
```bash
terraform stacks providers-lock
```

### "Missing required provider configuration"

If the state references a provider that was removed, add it back temporarily in `components.tfcomponent.hcl`, apply once to clean the state, then remove it in a follow-up upload.

### IAM AccessDenied errors

Check the inline policy on `hcp-terraform-stacks-role`:
```bash
aws iam get-role-policy --role-name hcp-terraform-stacks-role --policy-name stacks-workshop-policy
```

### "deployment_group" errors

Terraform Stacks does not support `deployment_group` with a `deployments` argument. Use `deployment_auto_approve` with `context.plan.deployment.deployment_name` conditions instead.

## File Structure

```
infrastructure/
├── components.tfcomponent.hcl    # Providers and component definition
├── variables.tfcomponent.hcl     # Stack-level input variables
├── outputs.tfcomponent.hcl       # Stack outputs per deployment
├── deployments.tfdeploy.hcl      # Deployments (dev, prod) and auto-approve rules
├── .terraform-version            # Required Terraform version (1.15.7)
├── .terraform.lock.hcl           # Provider lockfile (committed to VCS)
└── modules/
    └── workshop/                 # Infrastructure module
        ├── main.tf
        ├── ddb.tf
        ├── lambda-users.tf
        ├── api-gateway.tf
        ├── cognito.tf
        ├── lambda-authorizer.tf
        ├── variables.tf
        ├── outputs.tf
        └── placeholder.zip
```
