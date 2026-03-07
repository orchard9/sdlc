#!/usr/bin/env bash
# Bootstrap Gitea for local hub development.
# Idempotent — safe to run multiple times.
#
# Usage: ./dev/setup-gitea.sh [gitea-url]
#   Default gitea-url: http://localhost:3000

set -euo pipefail

GITEA_URL="${1:-http://localhost:7782}"
ADMIN_USER="sdlc-admin"
ADMIN_PASS="sdlc-admin"
ADMIN_EMAIL="admin@localhost"
ORG_NAME="orchard9"

echo "Gitea URL: $GITEA_URL"

# Wait for Gitea to be ready
echo "Waiting for Gitea..."
for i in $(seq 1 30); do
  if curl -sf "$GITEA_URL/api/v1/version" > /dev/null 2>&1; then
    echo "Gitea is ready."
    break
  fi
  if [ "$i" -eq 30 ]; then
    echo "ERROR: Gitea not reachable after 30s at $GITEA_URL" >&2
    exit 1
  fi
  sleep 1
done

# Create admin user via gitea CLI in the container (most reliable for fresh installs)
echo "Creating admin user '$ADMIN_USER'..."
docker compose exec -T -u git gitea gitea admin user create \
  --admin \
  --username "$ADMIN_USER" \
  --password "$ADMIN_PASS" \
  --email "$ADMIN_EMAIL" \
  --config /data/gitea/conf/app.ini \
  --must-change-password=false 2>/dev/null || echo "(user may already exist)"
echo "Admin user ready."

# Generate API token (delete existing first for idempotency)
echo "Generating API token..."
curl -sf -X DELETE \
  "$GITEA_URL/api/v1/users/$ADMIN_USER/tokens/sdlc-dev" \
  -u "$ADMIN_USER:$ADMIN_PASS" 2>/dev/null || true

TOKEN_RESP=$(curl -sf \
  "$GITEA_URL/api/v1/users/$ADMIN_USER/tokens" \
  -u "$ADMIN_USER:$ADMIN_PASS" \
  -H "Content-Type: application/json" \
  -d '{"name":"sdlc-dev","scopes":["all"]}' 2>/dev/null || echo "{}")

TOKEN=$(echo "$TOKEN_RESP" | grep -o '"sha1":"[^"]*"' | cut -d'"' -f4)
if [ -z "$TOKEN" ]; then
  echo "ERROR: Failed to generate API token. Response: $TOKEN_RESP" >&2
  exit 1
fi
echo "API Token: $TOKEN"

# Create orchard9 org
echo "Creating org '$ORG_NAME'..."
curl -sf -o /dev/null \
  "$GITEA_URL/api/v1/orgs" \
  -u "$ADMIN_USER:$ADMIN_PASS" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$ORG_NAME\",\"visibility\":\"public\"}" 2>/dev/null || echo "(org may already exist)"
echo "Org ready."

echo ""
echo "============================================"
echo "  Local Gitea setup complete!"
echo "============================================"
echo ""
echo "Add to your environment or docker-compose.env:"
echo ""
echo "  GITEA_URL=$GITEA_URL"
echo "  GITEA_API_TOKEN=$TOKEN"
echo "  DATABASE_URL=postgres://sdlc:sdlc@localhost:7781/sdlc"
echo "  SDLC_HUB=true"
echo "  SDLC_ROOT=/tmp/sdlc-hub"
echo ""
echo "Then start the hub:"
echo "  cargo watch -x 'run --bin ponder -- ui start --port 3141 --no-open --no-tunnel --hub'"
echo ""
