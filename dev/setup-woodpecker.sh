#!/usr/bin/env bash
# Bootstrap Woodpecker CI for local hub development.
# Idempotent — safe to run multiple times.
#
# Prerequisites:
#   - docker compose services running (postgres, gitea, woodpecker-server, woodpecker-agent)
#   - Gitea already bootstrapped (./dev/setup-gitea.sh)
#   - orchard9/sdlc repo exists in Gitea with a .woodpecker.yaml file
#
# Usage: ./dev/setup-woodpecker.sh
#
# What this does:
#   1. Creates a Gitea OAuth2 app for Woodpecker (if not exists)
#   2. Runs the Woodpecker OAuth flow programmatically to get a session
#   3. Generates a Woodpecker API token
#   4. Activates the orchard9/sdlc repo in Woodpecker
#   5. Prints env vars for the hub server

set -euo pipefail

GITEA_URL="${GITEA_URL:-http://localhost:7782}"
WP_URL="${WP_URL:-http://localhost:7783}"
ADMIN_USER="sdlc-admin"
ADMIN_PASS="sdlc-admin"
COOKIE_JAR=$(mktemp)
trap 'rm -f "$COOKIE_JAR"' EXIT

echo "Gitea:      $GITEA_URL"
echo "Woodpecker: $WP_URL"

# --- Wait for services ---
echo "Waiting for Woodpecker..."
for i in $(seq 1 30); do
  if curl -sf "$WP_URL/healthz" > /dev/null 2>&1; then
    echo "Woodpecker is ready."
    break
  fi
  if [ "$i" -eq 30 ]; then
    echo "ERROR: Woodpecker not reachable after 30s at $WP_URL" >&2
    exit 1
  fi
  sleep 1
done

# --- Step 1: Ensure Gitea OAuth2 app exists ---
echo "Checking Gitea OAuth2 apps..."
EXISTING_APP=$(curl -sf "$GITEA_URL/api/v1/user/applications/oauth2" \
  -u "$ADMIN_USER:$ADMIN_PASS" 2>/dev/null | \
  python3 -c "
import sys, json
apps = json.loads(sys.stdin.read())
for a in apps:
    if a['name'] == 'woodpecker':
        print(a['client_id'])
        break
" 2>/dev/null || echo "")

if [ -n "$EXISTING_APP" ]; then
  CLIENT_ID="$EXISTING_APP"
  echo "OAuth app exists (client_id: $CLIENT_ID)"
  echo "NOTE: If Woodpecker was configured with a different client_id, update docker-compose.yml"
else
  echo "Creating Gitea OAuth2 app for Woodpecker..."
  APP_RESP=$(curl -sf "$GITEA_URL/api/v1/user/applications/oauth2" \
    -u "$ADMIN_USER:$ADMIN_PASS" \
    -H "Content-Type: application/json" \
    -d "{
      \"name\": \"woodpecker\",
      \"redirect_uris\": [\"$WP_URL/authorize\"],
      \"confidential_client\": true
    }")
  CLIENT_ID=$(echo "$APP_RESP" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())['client_id'])")
  CLIENT_SECRET=$(echo "$APP_RESP" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())['client_secret'])")
  echo "Created OAuth app:"
  echo "  WOODPECKER_GITEA_CLIENT=$CLIENT_ID"
  echo "  WOODPECKER_GITEA_SECRET=$CLIENT_SECRET"
  echo ""
  echo "UPDATE docker-compose.yml with these values, then restart:"
  echo "  docker compose up -d --force-recreate woodpecker-server"
  echo "  Then re-run this script."
  exit 0
fi

# --- Step 2: Woodpecker OAuth flow ---
echo "Running Woodpecker OAuth flow..."

# 2a: Get state JWT from Woodpecker /authorize redirect
REDIRECT=$(curl -sS -D- -o /dev/null "$WP_URL/authorize" 2>&1 | grep -i "^location:" | tr -d '\r' | awk '{print $2}')
STATE=$(echo "$REDIRECT" | python3 -c "from urllib.parse import urlparse, parse_qs; import sys; print(parse_qs(urlparse(sys.stdin.read().strip()).query)['state'][0])")

# 2b: Login to Gitea
curl -sS -o /dev/null -c "$COOKIE_JAR" "$GITEA_URL/user/login" 2>/dev/null
CSRF=$(curl -sS -b "$COOKIE_JAR" "$GITEA_URL/user/login" 2>/dev/null | grep -o 'name="_csrf" content="[^"]*"' | head -1 | cut -d'"' -f4)
curl -sS -o /dev/null -b "$COOKIE_JAR" -c "$COOKIE_JAR" \
  "$GITEA_URL/user/login" \
  -d "_csrf=$CSRF&user_name=$ADMIN_USER&password=$ADMIN_PASS" 2>/dev/null

# 2c: Authorize (Gitea auto-grants after first time)
AUTH_RESP=$(curl -sS -D- -o /dev/null -b "$COOKIE_JAR" \
  "$GITEA_URL/login/oauth/authorize?client_id=$CLIENT_ID&redirect_uri=$(python3 -c "from urllib.parse import quote; print(quote('$WP_URL/authorize'))")&response_type=code&state=$STATE" 2>&1)

# Extract code from redirect Location header
CODE=$(echo "$AUTH_RESP" | grep -i "^location:" | tr -d '\r' | python3 -c "from urllib.parse import urlparse, parse_qs; import sys; print(parse_qs(urlparse(sys.stdin.read().strip().split(' ',1)[-1]).query)['code'][0])")

if [ -z "$CODE" ]; then
  # First time — need to grant. Get CSRF and POST the grant form.
  GRANT_CSRF=$(curl -sS -b "$COOKIE_JAR" \
    "$GITEA_URL/login/oauth/authorize?client_id=$CLIENT_ID&redirect_uri=$(python3 -c "from urllib.parse import quote; print(quote('$WP_URL/authorize'))")&response_type=code&state=$STATE" 2>/dev/null | \
    grep -o 'name="_csrf" value="[^"]*"' | head -1 | cut -d'"' -f4)
  AUTH_RESP=$(curl -sS -D- -o /dev/null -b "$COOKIE_JAR" -c "$COOKIE_JAR" \
    "$GITEA_URL/login/oauth/grant" \
    -d "_csrf=$GRANT_CSRF&client_id=$CLIENT_ID&redirect_uri=$(python3 -c "from urllib.parse import quote; print(quote('$WP_URL/authorize'))")&response_type=code&state=$STATE&granted=true" 2>&1)
  CODE=$(echo "$AUTH_RESP" | grep -i "^location:" | tr -d '\r' | python3 -c "from urllib.parse import urlparse, parse_qs; import sys; print(parse_qs(urlparse(sys.stdin.read().strip().split(' ',1)[-1]).query)['code'][0])")
fi

if [ -z "$CODE" ]; then
  echo "ERROR: Failed to get OAuth code from Gitea" >&2
  exit 1
fi

# 2d: Exchange code at Woodpecker
WP_COOKIE=$(curl -sS -D- -o /dev/null "$WP_URL/authorize?code=$CODE&state=$STATE" 2>&1 | \
  grep -i "^set-cookie: user_sess=" | head -1 | sed 's/^[Ss]et-[Cc]ookie: //' | cut -d';' -f1 | cut -d'=' -f2-)
if [ -z "$WP_COOKIE" ]; then
  echo "ERROR: Failed to get Woodpecker session" >&2
  exit 1
fi
echo "Woodpecker session obtained."

# --- Step 3: Generate API token ---
echo "Generating Woodpecker API token..."
WP_CSRF=$(curl -sS -b "user_sess=$WP_COOKIE" "$WP_URL/web-config.js" | \
  python3 -c "import sys,re; m=re.search(r'WOODPECKER_CSRF = \"([^\"]+)\"', sys.stdin.read()); print(m.group(1) if m else '')")

WP_TOKEN=$(curl -sS -X POST "$WP_URL/api/user/token" \
  -b "user_sess=$WP_COOKIE" \
  -H "X-CSRF-TOKEN: $WP_CSRF" \
  -H "Content-Type: application/json" \
  -d '{"name":"sdlc-hub"}')

if [ -z "$WP_TOKEN" ] || echo "$WP_TOKEN" | grep -q "error"; then
  echo "ERROR: Failed to generate API token: $WP_TOKEN" >&2
  exit 1
fi
echo "API token generated."

# --- Step 4: Activate orchard9/sdlc repo ---
echo "Activating orchard9/sdlc repo in Woodpecker..."

# Check if already activated
REPO_CHECK=$(curl -sS "$WP_URL/api/repos/lookup/orchard9/sdlc" \
  -H "Authorization: Bearer $WP_TOKEN" 2>/dev/null)
REPO_ACTIVE=$(echo "$REPO_CHECK" | python3 -c "import sys,json; print(json.loads(sys.stdin.read()).get('active', False))" 2>/dev/null || echo "false")

if [ "$REPO_ACTIVE" = "True" ]; then
  echo "Repo already activated."
  REPO_ID=$(echo "$REPO_CHECK" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())['id'])")
else
  # Get forge_remote_id from Gitea
  FORGE_ID=$(curl -sf "$GITEA_URL/api/v1/repos/orchard9/sdlc" \
    -u "$ADMIN_USER:$ADMIN_PASS" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())['id'])")

  ACTIVATE_RESP=$(curl -sS -D- -X POST "$WP_URL/api/repos?forge_remote_id=$FORGE_ID" \
    -H "Authorization: Bearer $WP_TOKEN" 2>&1)

  if echo "$ACTIVATE_RESP" | grep -q "HTTP.*200\|HTTP.*201"; then
    REPO_ID=$(echo "$ACTIVATE_RESP" | tail -1 | python3 -c "import sys,json; print(json.loads(sys.stdin.read())['id'])" 2>/dev/null || echo "?")
    echo "Repo activated (id: $REPO_ID)."
  else
    echo "WARN: Repo activation returned unexpected response."
    echo "This may fail if WOODPECKER_HOST is not reachable from Gitea."
    echo "The repo may still work if it was manually configured."
    REPO_ID="?"
  fi
fi

# --- Step 5: Verify with a test pipeline ---
echo ""
echo "Testing pipeline trigger..."
REPO_ID=$(curl -sS "$WP_URL/api/repos/lookup/orchard9/sdlc" \
  -H "Authorization: Bearer $WP_TOKEN" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())['id'])")

TEST_RESP=$(curl -sS -X POST "$WP_URL/api/repos/$REPO_ID/pipelines" \
  -H "Authorization: Bearer $WP_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"branch":"main","variables":{"PROVISION_SLUG":"setup-test"}}')

PIPELINE_STATUS=$(echo "$TEST_RESP" | python3 -c "import sys,json; p=json.loads(sys.stdin.read()); print(f'Pipeline #{p[\"number\"]}: {p[\"status\"]}')" 2>/dev/null || echo "Failed to parse response")
echo "$PIPELINE_STATUS"

echo ""
echo "============================================"
echo "  Woodpecker setup complete!"
echo "============================================"
echo ""
echo "Add to your hub server environment:"
echo ""
echo "  WOODPECKER_URL=$WP_URL"
echo "  WOODPECKER_API_TOKEN=$WP_TOKEN"
echo ""
echo "NOTE: Woodpecker tokens are invalidated on server restart."
echo "Re-run this script after 'docker compose restart woodpecker-server'."
echo ""
