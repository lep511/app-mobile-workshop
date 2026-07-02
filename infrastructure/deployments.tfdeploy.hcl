identity_token "aws" {
  audience = ["aws.workload.identity"]
}

deployment "dev" {
  inputs = {
    environment              = "dev"
    workshop_stack_base_name = "workshop-dev"
    role_arn                 = "arn:aws:iam::375920412105:role/hcp-terraform-stacks-role"
    identity_token           = identity_token.aws.jwt
  }
}

deployment "prod" {
  inputs = {
    environment              = "prod"
    workshop_stack_base_name = "workshop-prod"
    role_arn                 = "arn:aws:iam::375920412105:role/hcp-terraform-stacks-role"
    identity_token           = identity_token.aws.jwt
  }
}

deployment_auto_approve "dev_only" {
  check {
    condition = context.plan.deployment.deployment_name == "dev"
    reason    = "Only dev is auto-approved."
  }
}
