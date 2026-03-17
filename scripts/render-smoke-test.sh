#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  BACKEND_URL=https://api.nodrakeinthe.house FRONTEND_URL=https://nodrakeinthe.house ./scripts/render-smoke-test.sh

Optional auth flow:
  SMOKE_EMAIL=test-user@example.com SMOKE_PASSWORD='StrongPass123!' ./scripts/render-smoke-test.sh

Optional signup + auth flow:
  SMOKE_REGISTER=true ./scripts/render-smoke-test.sh
  SMOKE_REGISTER=true SMOKE_EMAIL=render-smoke@example.com SMOKE_PASSWORD='StrongPass123!' ./scripts/render-smoke-test.sh

Optional CORS overrides:
  ALLOWED_ORIGIN=https://nodrakeinthe.house DISALLOWED_ORIGIN=https://evil.example ./scripts/render-smoke-test.sh
EOF
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

BACKEND_URL="${BACKEND_URL:-}"
FRONTEND_URL="${FRONTEND_URL:-}"
SMOKE_EMAIL="${SMOKE_EMAIL:-}"
SMOKE_PASSWORD="${SMOKE_PASSWORD:-}"
SMOKE_REGISTER="${SMOKE_REGISTER:-false}"
DISALLOWED_ORIGIN="${DISALLOWED_ORIGIN:-https://invalid.example.invalid}"

if [[ -z "${BACKEND_URL}" || -z "${FRONTEND_URL}" ]]; then
  usage
  exit 1
fi

BACKEND_URL="${BACKEND_URL%/}"
FRONTEND_URL="${FRONTEND_URL%/}"
ALLOWED_ORIGIN="${ALLOWED_ORIGIN:-$FRONTEND_URL}"
ALLOWED_ORIGIN="${ALLOWED_ORIGIN%/}"

require_cmd curl
require_cmd python3

tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

log() {
  printf '[info] %s\n' "$1"
}

pass() {
  printf '[pass] %s\n' "$1"
}

fail() {
  printf '[fail] %s\n' "$1" >&2
  exit 1
}

json_read() {
  local key_path="$1"
  python3 - "$key_path" <<'PY'
import json
import sys

key_path = sys.argv[1].split(".")
data = json.load(sys.stdin)
value = data
for part in key_path:
    if part.isdigit():
        value = value[int(part)]
    else:
        value = value[part]
if isinstance(value, bool):
    print("true" if value else "false")
elif value is None:
    print("")
else:
    print(value)
PY
}

assert_json_success() {
  local response="$1"
  local context="$2"
  local success
  success="$(printf '%s' "$response" | json_read success 2>/dev/null || true)"
  [[ "$success" == "true" ]] || fail "${context} did not return success=true"
}

http_body_contains() {
  local url="$1"
  local expected="$2"
  local body
  body="$(curl -fsS "$url")"
  printf '%s' "$body" | grep -Fq "$expected"
}

check_frontend_route() {
  local path="$1"
  local url="${FRONTEND_URL}${path}"
  http_body_contains "$url" "No Drake in the House" || fail "Frontend route ${path} did not return the SPA shell"
  pass "Frontend route ${path} rewrites to the SPA shell"
}

check_preflight() {
  local origin="$1"
  local should_allow="$2"
  local headers_file="${tmpdir}/headers-$(date +%s%N)"

  curl -sS -o /dev/null -D "$headers_file" \
    -X OPTIONS \
    -H "Origin: ${origin}" \
    -H "Access-Control-Request-Method: POST" \
    -H "Access-Control-Request-Headers: content-type" \
    "${BACKEND_URL}/api/v1/auth/login"

  tr -d '\r' < "$headers_file" > "${headers_file}.normalized"

  if [[ "$should_allow" == "true" ]]; then
    grep -qi "^access-control-allow-origin: ${origin}$" "${headers_file}.normalized" || fail "CORS preflight did not allow ${origin}"
    pass "CORS preflight allows ${origin}"
  else
    if grep -qi "^access-control-allow-origin: ${origin}$" "${headers_file}.normalized"; then
      fail "CORS preflight unexpectedly allowed ${origin}"
    fi
    pass "CORS preflight blocks ${origin}"
  fi
}

perform_login_flow() {
  local email="$1"
  local password="$2"
  local login_response profile_response refresh_response logout_response
  local access_token refresh_token refreshed_access_token

  log "Testing login flow for ${email}"
  login_response="$(curl -fsS \
    -H "Content-Type: application/json" \
    -X POST \
    -d "{\"email\":\"${email}\",\"password\":\"${password}\"}" \
    "${BACKEND_URL}/api/v1/auth/login")"
  assert_json_success "$login_response" "Login"
  access_token="$(printf '%s' "$login_response" | json_read data.access_token)"
  refresh_token="$(printf '%s' "$login_response" | json_read data.refresh_token)"
  [[ -n "$access_token" && -n "$refresh_token" ]] || fail "Login did not return both access and refresh tokens"
  pass "Login returned access and refresh tokens"

  profile_response="$(curl -fsS \
    -H "Authorization: Bearer ${access_token}" \
    "${BACKEND_URL}/api/v1/users/profile")"
  assert_json_success "$profile_response" "Profile fetch"
  pass "Authenticated profile fetch succeeded"

  refresh_response="$(curl -fsS \
    -H "Content-Type: application/json" \
    -X POST \
    -d "{\"refresh_token\":\"${refresh_token}\"}" \
    "${BACKEND_URL}/api/v1/auth/refresh")"
  assert_json_success "$refresh_response" "Token refresh"
  refreshed_access_token="$(printf '%s' "$refresh_response" | json_read data.access_token)"
  [[ -n "$refreshed_access_token" ]] || fail "Refresh flow did not return an access token"
  pass "Token refresh returned a new access token"

  logout_response="$(curl -fsS \
    -H "Authorization: Bearer ${refreshed_access_token}" \
    -X POST \
    "${BACKEND_URL}/api/v1/auth/logout")"
  assert_json_success "$logout_response" "Logout"
  pass "Logout succeeded"
}

perform_registration_flow() {
  local email="$1"
  local password="$2"
  local register_response logout_response access_token

  log "Testing signup flow for ${email}"
  register_response="$(curl -fsS \
    -H "Content-Type: application/json" \
    -X POST \
    -d "{\"email\":\"${email}\",\"password\":\"${password}\",\"confirm_password\":\"${password}\",\"terms_accepted\":true}" \
    "${BACKEND_URL}/api/v1/auth/register")"
  assert_json_success "$register_response" "Registration"
  pass "Registration succeeded"

  access_token="$(printf '%s' "$register_response" | json_read data.access_token 2>/dev/null || true)"
  if [[ -n "$access_token" ]]; then
    logout_response="$(curl -fsS \
      -H "Authorization: Bearer ${access_token}" \
      -X POST \
      "${BACKEND_URL}/api/v1/auth/logout")"
    assert_json_success "$logout_response" "Post-registration logout"
    pass "Post-registration logout succeeded"
  fi
}

log "Checking backend health endpoints"
curl -fsS "${BACKEND_URL}/health" >/dev/null
pass "Backend /health responded successfully"

curl -fsS "${BACKEND_URL}/health/ready" >/dev/null
pass "Backend /health/ready responded successfully"

curl -fsS "${BACKEND_URL}/metrics" | grep -Eq '^# (HELP|TYPE) ' || fail "Backend /metrics did not return Prometheus output"
pass "Backend /metrics returned Prometheus-formatted metrics"

curl -fsS "${BACKEND_URL}/api/v1/offenses/" >/dev/null
pass "Public read-only API request succeeded"

log "Checking frontend SPA routing"
check_frontend_route "/"
check_frontend_route "/auth/callback/google"
check_frontend_route "/artist/test-id"

log "Checking OAuth bounce route"
oauth_headers="${tmpdir}/oauth-headers"
curl -sS -o /dev/null -D "$oauth_headers" \
  "${BACKEND_URL}/auth/callback/google?state=render-smoke&code=render-smoke"
tr -d '\r' < "$oauth_headers" | grep -Fqi "location: ${FRONTEND_URL}/auth/callback/google?state=render-smoke&code=render-smoke" \
  || fail "Backend OAuth bounce route did not redirect to the frontend callback URL"
pass "Backend OAuth bounce route redirects to the frontend callback URL"

log "Checking CORS policy"
check_preflight "$ALLOWED_ORIGIN" true
check_preflight "$DISALLOWED_ORIGIN" false

if [[ "$SMOKE_REGISTER" == "true" ]]; then
  if [[ -z "$SMOKE_PASSWORD" ]]; then
    SMOKE_PASSWORD='RenderSmoke123!'
  fi
  if [[ -z "$SMOKE_EMAIL" ]]; then
    SMOKE_EMAIL="render-smoke-$(date +%s)@example.com"
  fi
  perform_registration_flow "$SMOKE_EMAIL" "$SMOKE_PASSWORD"
  perform_login_flow "$SMOKE_EMAIL" "$SMOKE_PASSWORD"
elif [[ -n "$SMOKE_EMAIL" && -n "$SMOKE_PASSWORD" ]]; then
  perform_login_flow "$SMOKE_EMAIL" "$SMOKE_PASSWORD"
else
  log "Skipping signup/login smoke flow. Set SMOKE_REGISTER=true or provide SMOKE_EMAIL and SMOKE_PASSWORD to enable it."
fi

pass "Render smoke checks completed successfully"
