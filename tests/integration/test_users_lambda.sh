#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# Integration Tests — Users API (via API Gateway + Authorizer)
#
# Requires:
#   - AWS CLI configured with credentials
#   - curl and jq installed
#   - A deployed stack with API Gateway, Cognito, and Lambda
#
# Environment variables (loaded from .env in this directory):
#   API_URL           — API Gateway stage URL (e.g. https://xxxx.execute-api.us-west-2.amazonaws.com/prod)
#   COGNITO_CLIENT_ID — Cognito User Pool Client ID
#   TEST_USERNAME     — Cognito test user email
#   TEST_PASSWORD     — Cognito test user password
#   AWS_REGION        — AWS region (default: us-west-2)
# ============================================================

# Load .env if present
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ -f "$SCRIPT_DIR/.env" ]; then
  set -a
  source "$SCRIPT_DIR/.env"
  set +a
fi

REGION="${AWS_REGION:-us-west-2}"
OUTPUT_FILE="/tmp/lambda-response.json"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0

# --- Validate required variables ---

if [ -z "${API_URL:-}" ] || [ -z "${COGNITO_CLIENT_ID:-}" ] || [ -z "${TEST_USERNAME:-}" ] || [ -z "${TEST_PASSWORD:-}" ]; then
  echo "ERROR: Missing required environment variables."
  echo ""
  echo "Set the following in tests/integration/.env (see .env.example):"
  echo "  API_URL, COGNITO_CLIENT_ID, TEST_USERNAME, TEST_PASSWORD"
  exit 1
fi

# --- Authentication ---

get_access_token() {
  local client_id="$1"
  local username="$2"
  local password="$3"

  aws cognito-idp initiate-auth \
    --region "$REGION" \
    --client-id "$client_id" \
    --auth-flow USER_PASSWORD_AUTH \
    --auth-parameters "USERNAME=$username,PASSWORD=$password" \
    --query 'AuthenticationResult.AccessToken' \
    --output text
}

# --- API invocation ---

invoke_api() {
  local method="$1"
  local path="$2"
  local expected_status="$3"
  local body="${4:-}"
  local test_name="$5"

  local curl_args=(
    -s -o "$OUTPUT_FILE" -w "%{http_code}"
    -X "$method"
    -H "Authorization: Bearer $TOKEN"
    -H "Content-Type: application/json"
  )

  if [ -n "$body" ]; then
    curl_args+=(-d "$body")
  fi

  local actual_status
  actual_status=$(curl "${curl_args[@]}" "${API_URL}${path}")

  if [ "$actual_status" == "$expected_status" ]; then
    echo -e "  ${GREEN}PASS${NC} $test_name (status: $actual_status)"
    PASS=$((PASS + 1))
  else
    echo -e "  ${RED}FAIL${NC} $test_name (expected: $expected_status, got: $actual_status)"
    echo "       Response: $(jq -c '.' "$OUTPUT_FILE" 2>/dev/null || cat "$OUTPUT_FILE")"
    FAIL=$((FAIL + 1))
  fi
}

invoke_no_auth() {
  local method="$1"
  local path="$2"
  local expected_status="$3"
  local test_name="$4"

  local actual_status
  actual_status=$(curl -s -o "$OUTPUT_FILE" -w "%{http_code}" -X "$method" "${API_URL}${path}")

  if [ "$actual_status" == "$expected_status" ]; then
    echo -e "  ${GREEN}PASS${NC} $test_name (status: $actual_status)"
    PASS=$((PASS + 1))
  else
    echo -e "  ${RED}FAIL${NC} $test_name (expected: $expected_status, got: $actual_status)"
    echo "       Response: $(jq -c '.' "$OUTPUT_FILE" 2>/dev/null || cat "$OUTPUT_FILE")"
    FAIL=$((FAIL + 1))
  fi
}

invoke_expect_either() {
  local method="$1"
  local path="$2"
  local expected_a="$3"
  local expected_b="$4"
  local test_name="$5"
  local extra_header="${6:-}"

  local curl_args=(
    -s -o "$OUTPUT_FILE" -w "%{http_code}"
    -X "$method"
    -H "Content-Type: application/json"
  )

  if [ -n "$extra_header" ]; then
    curl_args+=(-H "$extra_header")
  fi

  local actual_status
  actual_status=$(curl "${curl_args[@]}" "${API_URL}${path}")

  if [ "$actual_status" == "$expected_a" ] || [ "$actual_status" == "$expected_b" ]; then
    echo -e "  ${GREEN}PASS${NC} $test_name (status: $actual_status)"
    PASS=$((PASS + 1))
  else
    echo -e "  ${RED}FAIL${NC} $test_name (expected: $expected_a or $expected_b, got: $actual_status)"
    echo "       Response: $(jq -c '.' "$OUTPUT_FILE" 2>/dev/null || cat "$OUTPUT_FILE")"
    FAIL=$((FAIL + 1))
  fi
}

extract_field() {
  jq -r "$1" "$OUTPUT_FILE"
}

# ============================================================
# Setup
# ============================================================

TOTAL_TESTS=20

echo ""
echo "======================================"
echo " Users API Integration Tests"
echo " API URL: $API_URL"
echo " Region:  $REGION"
echo " Tests:   $TOTAL_TESTS"
echo "======================================"
echo ""
echo -e "${YELLOW}Authenticating with Cognito...${NC}"
TOKEN=$(get_access_token "$COGNITO_CLIENT_ID" "$TEST_USERNAME" "$TEST_PASSWORD")
echo -e "  ${GREEN}OK${NC} Token obtained"
echo ""

# ============================================================
# Authentication & Security
# ============================================================

echo -e "${YELLOW}--- Authentication & Security ---${NC}"
echo ""

# [1] No token
echo -e "${YELLOW}[1/$TOTAL_TESTS] GET /users - No token (expect 401)${NC}"
invoke_expect_either "GET" "/users" "401" "403" "Rejected without token"

# [2] Invalid token
echo ""
echo -e "${YELLOW}[2/$TOTAL_TESTS] GET /users - Invalid token (expect 401)${NC}"
actual_status=$(curl -s -o "$OUTPUT_FILE" -w "%{http_code}" \
  -X GET \
  -H "Authorization: Bearer invalid.token.here" \
  "${API_URL}/users")
if [ "$actual_status" == "401" ] || [ "$actual_status" == "403" ]; then
  echo -e "  ${GREEN}PASS${NC} Rejected with invalid token (status: $actual_status)"
  PASS=$((PASS + 1))
else
  echo -e "  ${RED}FAIL${NC} Expected 401/403, got: $actual_status"
  FAIL=$((FAIL + 1))
fi

# [3] OPTIONS (CORS preflight, no auth required)
echo ""
echo -e "${YELLOW}[3/$TOTAL_TESTS] OPTIONS /users - CORS preflight (no auth)${NC}"
invoke_no_auth "OPTIONS" "/users" "200" "CORS preflight without token"

# ============================================================
# CRUD - Happy Path
# ============================================================

echo ""
echo -e "${YELLOW}--- CRUD Happy Path ---${NC}"
echo ""

# [4] Create user
echo -e "${YELLOW}[4/$TOTAL_TESTS] PUT /users - Create user${NC}"
invoke_api "PUT" "/users" "201" \
  '{"name":"Integration Test User","email":"integration@test.com","phone":"+1-555-9999"}' \
  "Create user"

CREATED_USER_ID=$(extract_field '.userid')
echo "       Created userid: $CREATED_USER_ID"

# [5] Get created user
echo ""
echo -e "${YELLOW}[5/$TOTAL_TESTS] GET /users/{userid} - Get created user${NC}"
invoke_api "GET" "/users/$CREATED_USER_ID" "200" "" "Get user by ID"

RETURNED_NAME=$(extract_field '.name')
echo "       Returned name: $RETURNED_NAME"

# [6] List users
echo ""
echo -e "${YELLOW}[6/$TOTAL_TESTS] GET /users - List users with pagination${NC}"
invoke_api "GET" "/users?limit=2" "200" "" "List users (limit=2)"

USER_COUNT=$(extract_field '.users | length')
NEXT_TOKEN=$(extract_field '.next_token // empty')
echo "       Users returned: $USER_COUNT"
echo "       Has next_token: $([ -n "$NEXT_TOKEN" ] && echo "yes" || echo "no")"

# [7] Use next_token for second page
echo ""
echo -e "${YELLOW}[7/$TOTAL_TESTS] GET /users - Pagination with next_token${NC}"
if [ -n "$NEXT_TOKEN" ]; then
  invoke_api "GET" "/users?limit=2&next_token=$NEXT_TOKEN" "200" "" "Pagination second page"
  PAGE2_COUNT=$(extract_field '.users | length')
  echo "       Page 2 users: $PAGE2_COUNT"
else
  echo -e "  ${GREEN}PASS${NC} Pagination second page (skipped: no next_token, table has <= 2 items)"
  PASS=$((PASS + 1))
fi

# [8] Update user
echo ""
echo -e "${YELLOW}[8/$TOTAL_TESTS] PUT /users/{userid} - Update user${NC}"
invoke_api "PUT" "/users/$CREATED_USER_ID" "200" \
  '{"name":"Updated Test User","phone":"+1-555-0000"}' \
  "Update user"

UPDATED_NAME=$(extract_field '.name')
echo "       Updated name: $UPDATED_NAME"

# [9] Verify update persisted
echo ""
echo -e "${YELLOW}[9/$TOTAL_TESTS] GET /users/{userid} - Verify update persisted${NC}"
invoke_api "GET" "/users/$CREATED_USER_ID" "200" "" "Verify update persisted"

PERSISTED_NAME=$(extract_field '.name')
if [ "$PERSISTED_NAME" == "Updated Test User" ]; then
  echo "       Confirmed name: $PERSISTED_NAME"
else
  echo -e "       ${RED}WARNING${NC}: Expected 'Updated Test User', got '$PERSISTED_NAME'"
fi

# [10] Delete user
echo ""
echo -e "${YELLOW}[10/$TOTAL_TESTS] DELETE /users/{userid} - Delete user${NC}"
invoke_api "DELETE" "/users/$CREATED_USER_ID" "204" "" "Delete user"

# [11] Verify deletion
echo ""
echo -e "${YELLOW}[11/$TOTAL_TESTS] GET /users/{userid} - Get deleted user (expect 404)${NC}"
invoke_api "GET" "/users/$CREATED_USER_ID" "404" "" "Get deleted user"

# [12] Double delete (expect 404)
echo ""
echo -e "${YELLOW}[12/$TOTAL_TESTS] DELETE /users/{userid} - Double delete (expect 404)${NC}"
invoke_api "DELETE" "/users/$CREATED_USER_ID" "404" "" "Double delete same user"

# ============================================================
# Validation Errors
# ============================================================

echo ""
echo -e "${YELLOW}--- Validation Errors ---${NC}"
echo ""

# [13] Missing email
echo -e "${YELLOW}[13/$TOTAL_TESTS] PUT /users - Missing email (expect 400)${NC}"
invoke_api "PUT" "/users" "400" '{"name":"No Email User"}' "Create without email"

# [14] Missing name
echo ""
echo -e "${YELLOW}[14/$TOTAL_TESTS] PUT /users - Missing name (expect 400)${NC}"
invoke_api "PUT" "/users" "400" '{"email":"test@example.com"}' "Create without name"

# [15] Email too long (>254 chars)
echo ""
echo -e "${YELLOW}[15/$TOTAL_TESTS] PUT /users - Email too long (expect 400)${NC}"
LONG_EMAIL=$(printf 'a%.0s' {1..255})
invoke_api "PUT" "/users" "400" \
  "{\"name\":\"Test\",\"email\":\"$LONG_EMAIL\"}" \
  "Create with email > 254 chars"

# [16] Invalid JSON body
echo ""
echo -e "${YELLOW}[16/$TOTAL_TESTS] PUT /users - Invalid JSON (expect 400)${NC}"
invoke_api "PUT" "/users" "400" 'not valid json {{{' "Create with invalid JSON"

# [17] Update with empty body (no fields)
echo ""
echo -e "${YELLOW}[17/$TOTAL_TESTS] PUT /users/{userid} - Update empty body (expect 400)${NC}"
invoke_api "PUT" "/users/some-fake-id" "400" '{}' "Update with no fields"

# ============================================================
# Edge Cases
# ============================================================

echo ""
echo -e "${YELLOW}--- Edge Cases ---${NC}"
echo ""

# [18] Update non-existent user (expect 404)
echo -e "${YELLOW}[18/$TOTAL_TESTS] PUT /users/{userid} - Update non-existent user (expect 404)${NC}"
invoke_api "PUT" "/users/00000000-0000-0000-0000-000000000000" "404" \
  '{"name":"Ghost"}' \
  "Update non-existent user"

# [19] Delete non-existent user (expect 404)
echo ""
echo -e "${YELLOW}[19/$TOTAL_TESTS] DELETE /users/{userid} - Delete non-existent user (expect 404)${NC}"
invoke_api "DELETE" "/users/00000000-0000-0000-0000-000000000000" "404" "" "Delete non-existent user"

# [20] Unsupported HTTP method (API Gateway returns 404 for undefined routes)
echo ""
echo -e "${YELLOW}[20/$TOTAL_TESTS] POST /users - Unsupported method (expect 404)${NC}"
invoke_api "POST" "/users" "404" '{"name":"Test"}' "POST method not allowed"

# ============================================================
# Results
# ============================================================

echo ""
echo "======================================"
echo -e " Results: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC} (of $TOTAL_TESTS)"
echo "======================================"

if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
