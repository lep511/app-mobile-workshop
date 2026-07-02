#!/usr/bin/env bash
set -euo pipefail

FUNCTION_NAME="${FUNCTION_NAME:-workshop-dev-users}"
REGION="${AWS_REGION:-us-west-2}"
OUTPUT_FILE="/tmp/lambda-response.json"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0

invoke_lambda() {
  local test_name="$1"
  local payload="$2"
  local expected_status="$3"

  aws lambda invoke \
    --function-name "$FUNCTION_NAME" \
    --region "$REGION" \
    --payload "$payload" \
    --cli-binary-format raw-in-base64-out \
    "$OUTPUT_FILE" > /dev/null 2>&1

  local actual_status
  actual_status=$(jq -r '.statusCode' "$OUTPUT_FILE")

  if [ "$actual_status" == "$expected_status" ]; then
    echo -e "  ${GREEN}PASS${NC} $test_name (status: $actual_status)"
    PASS=$((PASS + 1))
  else
    echo -e "  ${RED}FAIL${NC} $test_name (expected: $expected_status, got: $actual_status)"
    echo "       Response: $(jq -c '.body' "$OUTPUT_FILE")"
    FAIL=$((FAIL + 1))
  fi
}

extract_field() {
  jq -r ".body" "$OUTPUT_FILE" | jq -r "$1"
}

echo ""
echo "======================================"
echo " Users Lambda Integration Tests"
echo " Function: $FUNCTION_NAME"
echo " Region:   $REGION"
echo "======================================"
echo ""

# ------------------------------------------
echo -e "${YELLOW}[1/7] POST /users - Create user${NC}"
invoke_lambda "Create user" '{
  "version": "2.0",
  "routeKey": "PUT /users",
  "rawPath": "/users",
  "rawQueryString": "",
  "headers": {"content-type": "application/json"},
  "requestContext": {
    "accountId": "123456789012",
    "apiId": "test",
    "http": {"method": "PUT", "path": "/users", "protocol": "HTTP/1.1", "sourceIp": "127.0.0.1"},
    "requestId": "test-create",
    "routeKey": "PUT /users",
    "stage": "prod"
  },
  "body": "{\"name\":\"Integration Test User\",\"email\":\"integration@test.com\",\"phone\":\"+1-555-9999\"}",
  "isBase64Encoded": false
}' "201"

CREATED_USER_ID=$(extract_field '.userid')
echo "       Created userid: $CREATED_USER_ID"

# ------------------------------------------
echo ""
echo -e "${YELLOW}[2/7] GET /users/{userid} - Get created user${NC}"
invoke_lambda "Get user by ID" "{
  \"version\": \"2.0\",
  \"routeKey\": \"GET /users/{userid}\",
  \"rawPath\": \"/users/$CREATED_USER_ID\",
  \"rawQueryString\": \"\",
  \"headers\": {\"accept\": \"application/json\"},
  \"pathParameters\": {\"userid\": \"$CREATED_USER_ID\"},
  \"requestContext\": {
    \"accountId\": \"123456789012\",
    \"apiId\": \"test\",
    \"http\": {\"method\": \"GET\", \"path\": \"/users/$CREATED_USER_ID\", \"protocol\": \"HTTP/1.1\", \"sourceIp\": \"127.0.0.1\"},
    \"requestId\": \"test-get\",
    \"routeKey\": \"GET /users/{userid}\",
    \"stage\": \"dev\"
  },
  \"isBase64Encoded\": false
}" "200"

RETURNED_NAME=$(extract_field '.name')
echo "       Returned name: $RETURNED_NAME"

# ------------------------------------------
echo ""
echo -e "${YELLOW}[3/7] GET /users - List users with pagination${NC}"
invoke_lambda "List users (limit=2)" '{
  "version": "2.0",
  "routeKey": "GET /users",
  "rawPath": "/users",
  "rawQueryString": "limit=2",
  "headers": {"accept": "application/json"},
  "queryStringParameters": {"limit": "2"},
  "requestContext": {
    "accountId": "123456789012",
    "apiId": "test",
    "http": {"method": "GET", "path": "/users", "protocol": "HTTP/1.1", "sourceIp": "127.0.0.1"},
    "requestId": "test-list",
    "routeKey": "GET /users",
    "stage": "dev"
  },
  "isBase64Encoded": false
}' "200"

USER_COUNT=$(extract_field '.users | length')
NEXT_TOKEN=$(extract_field '.next_token // empty')
echo "       Users returned: $USER_COUNT"
echo "       Has next_token: $([ -n "$NEXT_TOKEN" ] && echo "yes" || echo "no")"

# ------------------------------------------
echo ""
echo -e "${YELLOW}[4/7] PUT /users/{userid} - Update user${NC}"
invoke_lambda "Update user" "{
  \"version\": \"2.0\",
  \"routeKey\": \"PUT /users/{userid}\",
  \"rawPath\": \"/users/$CREATED_USER_ID\",
  \"rawQueryString\": \"\",
  \"headers\": {\"content-type\": \"application/json\"},
  \"pathParameters\": {\"userid\": \"$CREATED_USER_ID\"},
  \"requestContext\": {
    \"accountId\": \"123456789012\",
    \"apiId\": \"test\",
    \"http\": {\"method\": \"PUT\", \"path\": \"/users/$CREATED_USER_ID\", \"protocol\": \"HTTP/1.1\", \"sourceIp\": \"127.0.0.1\"},
    \"requestId\": \"test-update\",
    \"routeKey\": \"PUT /users/{userid}\",
    \"stage\": \"dev\"
  },
  \"body\": \"{\\\"name\\\":\\\"Updated Test User\\\",\\\"phone\\\":\\\"+1-555-0000\\\"}\",
  \"isBase64Encoded\": false
}" "200"

UPDATED_NAME=$(extract_field '.name')
echo "       Updated name: $UPDATED_NAME"

# ------------------------------------------
echo ""
echo -e "${YELLOW}[5/7] DELETE /users/{userid} - Delete user${NC}"
invoke_lambda "Delete user" "{
  \"version\": \"2.0\",
  \"routeKey\": \"DELETE /users/{userid}\",
  \"rawPath\": \"/users/$CREATED_USER_ID\",
  \"rawQueryString\": \"\",
  \"headers\": {\"accept\": \"application/json\"},
  \"pathParameters\": {\"userid\": \"$CREATED_USER_ID\"},
  \"requestContext\": {
    \"accountId\": \"123456789012\",
    \"apiId\": \"test\",
    \"http\": {\"method\": \"DELETE\", \"path\": \"/users/$CREATED_USER_ID\", \"protocol\": \"HTTP/1.1\", \"sourceIp\": \"127.0.0.1\"},
    \"requestId\": \"test-delete\",
    \"routeKey\": \"DELETE /users/{userid}\",
    \"stage\": \"dev\"
  },
  \"isBase64Encoded\": false
}" "204"

# ------------------------------------------
echo ""
echo -e "${YELLOW}[6/7] GET /users/{userid} - Get deleted user (expect 404)${NC}"
invoke_lambda "Get deleted user" "{
  \"version\": \"2.0\",
  \"routeKey\": \"GET /users/{userid}\",
  \"rawPath\": \"/users/$CREATED_USER_ID\",
  \"rawQueryString\": \"\",
  \"headers\": {\"accept\": \"application/json\"},
  \"pathParameters\": {\"userid\": \"$CREATED_USER_ID\"},
  \"requestContext\": {
    \"accountId\": \"123456789012\",
    \"apiId\": \"test\",
    \"http\": {\"method\": \"GET\", \"path\": \"/users/$CREATED_USER_ID\", \"protocol\": \"HTTP/1.1\", \"sourceIp\": \"127.0.0.1\"},
    \"requestId\": \"test-get-deleted\",
    \"routeKey\": \"GET /users/{userid}\",
    \"stage\": \"dev\"
  },
  \"isBase64Encoded\": false
}" "404"

# ------------------------------------------
echo ""
echo -e "${YELLOW}[7/7] PUT /users - Validation error (missing email)${NC}"
invoke_lambda "Create user without email" '{
  "version": "2.0",
  "routeKey": "PUT /users",
  "rawPath": "/users",
  "rawQueryString": "",
  "headers": {"content-type": "application/json"},
  "requestContext": {
    "accountId": "123456789012",
    "apiId": "test",
    "http": {"method": "PUT", "path": "/users", "protocol": "HTTP/1.1", "sourceIp": "127.0.0.1"},
    "requestId": "test-validation",
    "routeKey": "PUT /users",
    "stage": "prod"
  },
  "body": "{\"name\":\"No Email User\"}",
  "isBase64Encoded": false
}' "400"

# ------------------------------------------
echo ""
echo "======================================"
echo -e " Results: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}"
echo "======================================"

if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
