# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Serverless Patterns Workshop — an AWS serverless application with Rust Lambda functions (ARM64), Terraform infrastructure, and a REST API backed by DynamoDB with Cognito authentication.

## Tech Stack

- **Language:** Rust (stable toolchain, targets `aarch64-unknown-linux-gnu`)
- **Infrastructure:** Terraform (>= 1.0.0), deployed via HCP Terraform Stacks
- **Cloud:** AWS (Lambda `provided.al2023`, API Gateway HTTP API, DynamoDB, Cognito, CloudWatch, X-Ray)
- **Build tool:** cargo-lambda
- **Testing:** `cargo test` (integration tests in `tests/` directory per crate)

## Commands

```bash
# Infrastructure
cd infrastructure && terraform init
terraform plan
terraform apply
terraform apply -var="region=us-east-1"

# Build Lambda functions
./scripts/build-users-lambda.sh
./scripts/build-authorizer-lambda.sh

# Tests — users lambda
cd src/functions/users && cargo test

# Tests — authorizer lambda
cd src/functions/authorizer && cargo test

# Integration tests (requires deployed Lambda)
./tests/integration/test_users_lambda.sh
```

## Architecture

Two Rust Lambda functions behind API Gateway HTTP API with a custom authorizer:

- **Users service** (`src/functions/users/`) — CRUD operations with lib/bin split: `lib.rs` (public API), `main.rs` (Lambda entry point), `handlers.rs`, `models.rs`, `errors.rs`
- **Authorizer** (`src/functions/authorizer/`) — Custom Lambda authorizer: `lib.rs` (JWT validation logic), `main.rs` (Lambda entry point)

Infrastructure is in `infrastructure/modules/workshop/`: `api-gateway.tf`, `cognito.tf`, `ddb.tf`, `lambda-users.tf`, `lambda-authorizer.tf`.

Tests are in the `tests/` directory of each crate (Cargo integration test convention), with shared helpers in `tests/common/mod.rs`.

## Deployment

See [`infrastructure/HCP_TERRAFORM.md`](infrastructure/HCP_TERRAFORM.md) for full HCP Terraform Stacks deployment instructions (org: `lep511`, stack: `stack-mobile-app-v2`).
