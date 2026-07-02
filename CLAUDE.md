# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Serverless Patterns Workshop — an AWS serverless application with Python Lambda functions, Terraform infrastructure, and a REST API backed by DynamoDB with Cognito authentication.

## Tech Stack

- **Language:** Python 3.x
- **Infrastructure:** Terraform (>= 1.0.0)
- **Cloud:** AWS (Lambda, API Gateway, DynamoDB, Cognito, CloudWatch)
- **Testing:** pytest

## Commands

```bash
# Infrastructure
cd infrastructure && terraform init
terraform plan
terraform apply
terraform apply -var="region=us-east-1"

# Tests
pytest tests/unit/
pytest tests/integration/
pytest tests/unit/users/test_app.py  # single test file

# Dependencies
pip install -r src/users/requirements.txt
pip install -r src/users/requirements-dev.txt
```

## Architecture

Two Lambda functions behind API Gateway with a custom authorizer:

- **Users service** (`src/users/`) — CRUD operations, handler in `lambda_function.py`, backed by DynamoDB
- **Authorizer** (`src/authorizer/`) — Custom Lambda authorizer (`lambda_authorizer.py`) validating tokens from Cognito

Infrastructure is split by AWS service (`infrastructure/`): `api-gateway.tf`, `cognito.tf`, `ddb.tf`, `lambda.tf`, `lambda-authorizer.tf`, `monitoring.tf`. Shared config lives in `provider.tf`, `variables.tf`, `versions.tf`.

Tests mirror source structure: `tests/unit/users/` for unit tests, `tests/integration/` for integration tests. Fixtures go in `conftest.py`.

## Deployment

See [`infrastructure/HCP_TERRAFORM.md`](infrastructure/HCP_TERRAFORM.md) for full HCP Terraform Stacks deployment instructions (org: `lep511`, stack: `stack-mobile-app-v2`).
