required_providers {
  aws = {
    source  = "hashicorp/aws"
    version = "~> 6.52"
  }
}

provider "aws" "this" {
  config {
    region = var.region

    assume_role_with_web_identity {
      role_arn           = var.role_arn
      web_identity_token = var.identity_token
    }
  }
}

component "workshop" {
  source = "./modules/workshop"

  inputs = {
    workshop_stack_base_name = var.workshop_stack_base_name
    environment              = var.environment
    project                  = var.project
    cors_allowed_origins     = var.cors_allowed_origins
  }

  providers = {
    aws = provider.aws.this
  }
}
