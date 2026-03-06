# Desktop App Integration Guide

Complete guide for integrating desktop applications with a2api by reading CLI credentials.

## Overview

### What is Desktop App Integration?

Desktop apps can read the `~/.a2api` credentials file written by the a2api CLI to access the user's session without requiring separate authentication. This provides a seamless experience when your desktop app co-exists with the CLI.

### When to Use This Pattern

Use CLI credentials when:
- Building a desktop application (Electron, Tauri, native)
- App runs on the same machine as the a2api CLI
- Single user per machine
- Want to share the CLI's existing session

For web apps or multi-user scenarios, see [OAuth Integration](oauth.md).

### How It Works

```
User runs `a2api login` → CLI writes ~/.a2api →
→ Desktop app reads ~/.a2api → Extracts access token →
→ Calls proxy API
```

---

## Prerequisites

Before you begin:

- **CLI Installed:** User must have a2api CLI installed
- **User Logged In:** User must run `a2api login` at least once
- **Same Machine:** Your app must run on the same machine as the CLI
- **File Access:** Ability to read `~/.a2api` file
- **TOML Parser:** TOML parsing library for your language

---

## Step 1: Locate Credentials File

The CLI stores credentials in `~/.a2api` (TOML format).

### Platform-Specific Paths

| Platform | Path |
|----------|------|
| **Linux** | `~/.a2api` or `$HOME/.a2api` |
| **macOS** | `~/.a2api` or `$HOME/.a2api` |
| **Windows** | `%USERPROFILE%\.a2api` |

### Example: Get Path in Code

**Python:**
```python
import os
from pathlib import Path

credentials_path = Path.home() / '.a2api'
```

**Node.js:**
```javascript
const os = require('os');
const path = require('path');

const credentialsPath = path.join(os.homedir(), '.a2api');
```

**Go:**
```go
import (
    "os"
    "path/filepath"
)

homeDir, _ := os.UserHomeDir()
credentialsPath := filepath.Join(homeDir, ".a2api")
```

---

## Step 2: Parse TOML Credentials

The `~/.a2api` file uses TOML format with three sections: `[auth]`, `[user]`, `[gateway]`.

### File Format

```toml
[auth]
access_token = "a2api_access_01JH5YQRMGDCK8V7N2W3X4Z5A6B7C8D9E0F1G2H3I4J5K6"
refresh_token = "a2api_refresh_01JH5YQRMGDCK8V7N2W3X4Z5A6B7C8D9E0F1G2H3I4J5K6"
expires_at = "2025-01-15T11:30:00Z"

[user]
id = "user_01JH5YQRMGDCK8V7N2W3X4Z5A6"
email = "user@example.com"

[gateway]
api = "https://api.a2api.orchard9.ai"
web = "https://a2api.orchard9.ai"
```

### Field Descriptions

**[auth] Section:**

| Field | Description |
|-------|-------------|
| `access_token` | JWT token for API requests (expires in 1 hour) |
| `refresh_token` | Token to get new access tokens |
| `expires_at` | ISO 8601 timestamp when access token expires |

**[user] Section:**

| Field | Description |
|-------|-------------|
| `id` | User's unique ID |
| `email` | User's email address |

**[gateway] Section:**

| Field | Description |
|-------|-------------|
| `api` | a2api API base URL |
| `web` | a2api Web UI base URL |

### Parse TOML by Language

**Python:**

```python
import toml
from pathlib import Path

credentials_path = Path.home() / '.a2api'

# Check if file exists
if not credentials_path.exists():
    print("User must run 'a2api login' first")
    exit(1)

# Parse TOML
credentials = toml.load(credentials_path)

access_token = credentials['auth']['access_token']
refresh_token = credentials['auth']['refresh_token']
expires_at = credentials['auth']['expires_at']
api_url = credentials['gateway']['api']

print(f"Access token: {access_token}")
```

**Node.js:**

```javascript
const fs = require('fs');
const path = require('path');
const toml = require('toml');
const os = require('os');

const credentialsPath = path.join(os.homedir(), '.a2api');

// Check if file exists
if (!fs.existsSync(credentialsPath)) {
  console.error("User must run 'a2api login' first");
  process.exit(1);
}

// Parse TOML
const credentialsFile = fs.readFileSync(credentialsPath, 'utf-8');
const credentials = toml.parse(credentialsFile);

const accessToken = credentials.auth.access_token;
const refreshToken = credentials.auth.refresh_token;
const expiresAt = credentials.auth.expires_at;
const apiUrl = credentials.gateway.api;

console.log(`Access token: ${accessToken}`);
```

**Go:**

```go
package main

import (
    "fmt"
    "os"
    "path/filepath"

    "github.com/BurntSushi/toml"
)

type Credentials struct {
    Auth struct {
        AccessToken   string `toml:"access_token"`
        RefreshToken  string `toml:"refresh_token"`
        ExpiresAt     string `toml:"expires_at"`
    } `toml:"auth"`
    User struct {
        ID    string `toml:"id"`
        Email string `toml:"email"`
    } `toml:"user"`
    Gateway struct {
        API string `toml:"api"`
        Web string `toml:"web"`
    } `toml:"gateway"`
}

func main() {
    homeDir, _ := os.UserHomeDir()
    credentialsPath := filepath.Join(homeDir, ".a2api")

    // Check if file exists
    if _, err := os.Stat(credentialsPath); os.IsNotExist(err) {
        fmt.Println("User must run 'a2api login' first")
        os.Exit(1)
    }

    // Parse TOML
    var credentials Credentials
    if _, err := toml.DecodeFile(credentialsPath, &credentials); err != nil {
        fmt.Printf("Failed to parse credentials: %v\n", err)
        os.Exit(1)
    }

    fmt.Printf("Access token: %s\n", credentials.Auth.AccessToken)
}
```

---

## Step 3: Check Token Expiry

Access tokens expire after 1 hour. Check if the token needs refreshing before using it.

### Parse Expiry Timestamp

**Python:**

```python
from datetime import datetime, timedelta

expires_at = datetime.fromisoformat(credentials['auth']['expires_at'].replace('Z', '+00:00'))
now = datetime.now(expires_at.tzinfo)

# Refresh if expiring in < 1 minute
if expires_at < now + timedelta(minutes=1):
    print("Token expired or expiring soon - need to refresh")
    # See Step 4 for refresh logic
else:
    print("Token is valid")
```

**Node.js:**

```javascript
const expiresAt = new Date(credentials.auth.expires_at);
const now = new Date();

// Refresh if expiring in < 1 minute
const ONE_MINUTE = 60 * 1000;
if (expiresAt.getTime() < now.getTime() + ONE_MINUTE) {
  console.log("Token expired or expiring soon - need to refresh");
  // See Step 4 for refresh logic
} else {
  console.log("Token is valid");
}
```

**Go:**

```go
import "time"

expiresAt, _ := time.Parse(time.RFC3339, credentials.Auth.ExpiresAt)
now := time.Now()

// Refresh if expiring in < 1 minute
if expiresAt.Before(now.Add(1 * time.Minute)) {
    fmt.Println("Token expired or expiring soon - need to refresh")
    // See Step 4 for refresh logic
} else {
    fmt.Println("Token is valid")
}
```

---

## Step 4: Refresh Token (If Needed)

If the access token is expired or expiring soon, use the refresh token to get a new access token.

### API Endpoint

```http
POST https://api.a2api.orchard9.ai/auth/token
Content-Type: application/json

{
  "grant_type": "refresh_token",
  "refresh_token": "<refresh_token>"
}
```

### Response

```json
{
  "access_token": "a2api_access_NEW123...",
  "refresh_token": "a2api_refresh_NEW456...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### Token Rotation

⚠️ **Critical:** a2api uses token family rotation:

1. Each refresh returns **new** access token + **new** refresh token
2. Old refresh token is invalidated
3. **You must update `~/.a2api` with new tokens**
4. Reusing old refresh token revokes entire token family

### Example: Refresh Token

**Python:**

```python
import requests
import toml
from datetime import datetime, timedelta
from pathlib import Path

def refresh_access_token():
    credentials_path = Path.home() / '.a2api'
    credentials = toml.load(credentials_path)

    base_url = credentials['gateway']['api']
    refresh_token = credentials['auth']['refresh_token']

    # Call refresh endpoint
    response = requests.post(
        f"{base_url}/auth/token",
        json={
            "grant_type": "refresh_token",
            "refresh_token": refresh_token
        }
    )

    if response.status_code != 200:
        print("Failed to refresh token - user must re-login")
        exit(1)

    data = response.json()

    # Update credentials with NEW tokens
    credentials['auth']['access_token'] = data['access_token']
    credentials['auth']['refresh_token'] = data['refresh_token']

    # Calculate new expiry time
    expires_at = datetime.now() + timedelta(seconds=data['expires_in'])
    credentials['auth']['expires_at'] = expires_at.isoformat() + 'Z'

    # Write back to ~/.a2api
    with open(credentials_path, 'w') as f:
        toml.dump(credentials, f)

    return data['access_token']
```

**Node.js:**

```javascript
const axios = require('axios');
const fs = require('fs');
const toml = require('toml');
const tomlify = require('tomlify-j0.4');

async function refreshAccessToken() {
  const credentialsPath = path.join(os.homedir(), '.a2api');
  const credentialsFile = fs.readFileSync(credentialsPath, 'utf-8');
  const credentials = toml.parse(credentialsFile);

  const baseUrl = credentials.gateway.api;
  const refreshToken = credentials.auth.refresh_token;

  try {
    // Call refresh endpoint
    const response = await axios.post(`${baseUrl}/auth/token`, {
      grant_type: 'refresh_token',
      refresh_token: refreshToken
    });

    const { access_token, refresh_token: newRefreshToken, expires_in } = response.data;

    // Update credentials with NEW tokens
    credentials.auth.access_token = access_token;
    credentials.auth.refresh_token = newRefreshToken;

    // Calculate new expiry time
    const expiresAt = new Date(Date.now() + expires_in * 1000);
    credentials.auth.expires_at = expiresAt.toISOString();

    // Write back to ~/.a2api
    fs.writeFileSync(credentialsPath, tomlify.toToml(credentials, { space: 2 }));

    return access_token;
  } catch (error) {
    console.error('Failed to refresh token - user must re-login');
    process.exit(1);
  }
}
```

---

## Step 5: Use Access Token with Proxy API

Use the access token to proxy API requests, just like OAuth integration.

### API Endpoint

```http
POST https://api.a2api.orchard9.ai/proxy
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "connection_id": "<connection_id>",
  "request": {
    "method": "POST",
    "path": "/v1/chat/completions",
    "headers": {
      "Content-Type": "application/json"
    },
    "body": {
      "model": "gpt-4",
      "messages": [{"role": "user", "content": "Hello!"}]
    }
  }
}
```

### Example: Proxy OpenAI Request

**Python:**

```python
import requests
from pathlib import Path
import toml

def proxy_openai_request(connection_id, messages):
    credentials_path = Path.home() / '.a2api'
    credentials = toml.load(credentials_path)

    access_token = get_valid_access_token()  # See helper below
    base_url = credentials['gateway']['api']

    response = requests.post(
        f"{base_url}/proxy",
        headers={
            "Authorization": f"Bearer {access_token}"
        },
        json={
            "connection_id": connection_id,
            "request": {
                "method": "POST",
                "path": "/v1/chat/completions",
                "headers": {
                    "Content-Type": "application/json"
                },
                "body": {
                    "model": "gpt-4",
                    "messages": messages
                }
            }
        }
    )

    return response.json()

# Helper: Get valid access token (auto-refresh)
def get_valid_access_token():
    credentials_path = Path.home() / '.a2api'
    credentials = toml.load(credentials_path)

    expires_at = datetime.fromisoformat(credentials['auth']['expires_at'].replace('Z', '+00:00'))
    now = datetime.now(expires_at.tzinfo)

    # Refresh if expiring in < 1 minute
    if expires_at < now + timedelta(minutes=1):
        return refresh_access_token()

    return credentials['auth']['access_token']
```

**Node.js:**

```javascript
async function proxyOpenAIRequest(connectionId, messages) {
  const accessToken = await getValidAccessToken();  // See helper below
  const credentials = getCredentials();
  const baseUrl = credentials.gateway.api;

  const response = await axios.post(`${baseUrl}/proxy`, {
    connection_id: connectionId,
    request: {
      method: 'POST',
      path: '/v1/chat/completions',
      headers: {
        'Content-Type': 'application/json'
      },
      body: {
        model: 'gpt-4',
        messages
      }
    }
  }, {
    headers: {
      'Authorization': `Bearer ${accessToken}`
    }
  });

  return response.data;
}

// Helper: Get valid access token (auto-refresh)
async function getValidAccessToken() {
  const credentials = getCredentials();
  const expiresAt = new Date(credentials.auth.expires_at);
  const now = new Date();

  const ONE_MINUTE = 60 * 1000;
  if (expiresAt.getTime() < now.getTime() + ONE_MINUTE) {
    return await refreshAccessToken();
  }

  return credentials.auth.access_token;
}
```

---

## Security Checklist

### File Permissions

✅ **DO:**
- Verify `~/.a2api` has restrictive permissions (0600 on Unix)
- Only allow owner read/write access
- Warn user if permissions are too permissive

❌ **DON'T:**
- Allow world-readable credentials files
- Ignore permission errors

**Example: Check Permissions (Python):**

```python
import os
import stat

credentials_path = Path.home() / '.a2api'
file_stat = os.stat(credentials_path)
file_mode = stat.filemode(file_stat.st_mode)

# Check if file is readable by others
if file_stat.st_mode & (stat.S_IRGRP | stat.S_IROTH):
    print("WARNING: ~/.a2api has insecure permissions!")
    print("Run: chmod 600 ~/.a2api")
```

### Token Handling

✅ **DO:**
- Handle token rotation (store new tokens after refresh)
- Refresh tokens before expiry (1-minute buffer)
- Clear tokens from memory when done
- Handle token family revocation gracefully

❌ **DON'T:**
- Log tokens to console/files
- Reuse old refresh tokens (triggers revocation)
- Ignore token refresh errors
- Store tokens in insecure locations

### Error Handling

✅ **DO:**
- Check if `~/.a2api` exists before reading
- Handle TOML parse errors
- Handle refresh token expiry/revocation
- Prompt user to re-run `a2api login` when needed

❌ **DON'T:**
- Crash on missing credentials file
- Assume tokens are always valid
- Ignore refresh errors

---

## Complete Examples

### Python Desktop App (50 lines)

```python
import requests
import toml
from pathlib import Path
from datetime import datetime, timedelta
import sys

class A2APIClient:
    def __init__(self):
        self.credentials_path = Path.home() / '.a2api'

        if not self.credentials_path.exists():
            print("Error: User must run 'a2api login' first")
            sys.exit(1)

    def get_credentials(self):
        return toml.load(self.credentials_path)

    def is_token_expired(self):
        credentials = self.get_credentials()
        expires_at = datetime.fromisoformat(
            credentials['auth']['expires_at'].replace('Z', '+00:00')
        )
        now = datetime.now(expires_at.tzinfo)
        return expires_at < now + timedelta(minutes=1)

    def refresh_token(self):
        credentials = self.get_credentials()
        base_url = credentials['gateway']['api']

        response = requests.post(
            f"{base_url}/auth/token",
            json={
                "grant_type": "refresh_token",
                "refresh_token": credentials['auth']['refresh_token']
            }
        )

        if response.status_code != 200:
            print("Error: Token refresh failed - please re-login")
            sys.exit(1)

        data = response.json()
        credentials['auth']['access_token'] = data['access_token']
        credentials['auth']['refresh_token'] = data['refresh_token']
        expires_at = datetime.now() + timedelta(seconds=data['expires_in'])
        credentials['auth']['expires_at'] = expires_at.isoformat() + 'Z'

        with open(self.credentials_path, 'w') as f:
            toml.dump(credentials, f)

        return data['access_token']

    def get_access_token(self):
        if self.is_token_expired():
            return self.refresh_token()
        return self.get_credentials()['auth']['access_token']

    def proxy_request(self, connection_id, method, path, body=None):
        credentials = self.get_credentials()
        access_token = self.get_access_token()
        base_url = credentials['gateway']['api']

        response = requests.post(
            f"{base_url}/proxy",
            headers={"Authorization": f"Bearer {access_token}"},
            json={
                "connection_id": connection_id,
                "request": {
                    "method": method,
                    "path": path,
                    "headers": {"Content-Type": "application/json"},
                    "body": body
                }
            }
        )

        return response.json()

# Usage
client = A2APIClient()
result = client.proxy_request(
    connection_id="conn_123",
    method="POST",
    path="/v1/chat/completions",
    body={
        "model": "gpt-4",
        "messages": [{"role": "user", "content": "Hello!"}]
    }
)
print(result)
```

### Node.js Electron App (60 lines)

```javascript
const axios = require('axios');
const fs = require('fs');
const path = require('path');
const os = require('os');
const toml = require('toml');
const tomlify = require('tomlify-j0.4');

class A2APIClient {
  constructor() {
    this.credentialsPath = path.join(os.homedir(), '.a2api');

    if (!fs.existsSync(this.credentialsPath)) {
      throw new Error("User must run 'a2api login' first");
    }
  }

  getCredentials() {
    const file = fs.readFileSync(this.credentialsPath, 'utf-8');
    return toml.parse(file);
  }

  isTokenExpired() {
    const credentials = this.getCredentials();
    const expiresAt = new Date(credentials.auth.expires_at);
    const now = new Date();
    const ONE_MINUTE = 60 * 1000;

    return expiresAt.getTime() < now.getTime() + ONE_MINUTE;
  }

  async refreshToken() {
    const credentials = this.getCredentials();
    const baseUrl = credentials.gateway.api;

    try {
      const response = await axios.post(`${baseUrl}/auth/token`, {
        grant_type: 'refresh_token',
        refresh_token: credentials.auth.refresh_token
      });

      const { access_token, refresh_token, expires_in } = response.data;

      credentials.auth.access_token = access_token;
      credentials.auth.refresh_token = refresh_token;
      const expiresAt = new Date(Date.now() + expires_in * 1000);
      credentials.auth.expires_at = expiresAt.toISOString();

      fs.writeFileSync(this.credentialsPath, tomlify.toToml(credentials, { space: 2 }));

      return access_token;
    } catch (error) {
      throw new Error('Token refresh failed - please re-login');
    }
  }

  async getAccessToken() {
    if (this.isTokenExpired()) {
      return await this.refreshToken();
    }
    return this.getCredentials().auth.access_token;
  }

  async proxyRequest(connectionId, method, path, body = null) {
    const credentials = this.getCredentials();
    const accessToken = await this.getAccessToken();
    const baseUrl = credentials.gateway.api;

    const response = await axios.post(`${baseUrl}/proxy`, {
      connection_id: connectionId,
      request: {
        method,
        path,
        headers: { 'Content-Type': 'application/json' },
        body
      }
    }, {
      headers: { 'Authorization': `Bearer ${accessToken}` }
    });

    return response.data;
  }
}

// Usage
(async () => {
  const client = new A2APIClient();
  const result = await client.proxyRequest(
    'conn_123',
    'POST',
    '/v1/chat/completions',
    {
      model: 'gpt-4',
      messages: [{ role: 'user', content: 'Hello!' }]
    }
  );
  console.log(result);
})();
```

---

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| File not found | User hasn't run `a2api login` | Prompt user to run `a2api login` |
| TOML parse error | Corrupted credentials file | Prompt user to re-run `a2api login` |
| 401 Unauthorized | Access token expired | Refresh token (see Step 4) |
| Token refresh failed | Refresh token expired/revoked | Prompt user to re-run `a2api login` |
| Token family revoked | Old refresh token reused (security) | Prompt user to re-run `a2api login` |

### Handle Missing Credentials

```python
if not Path.home().joinpath('.a2api').exists():
    print("Error: Please run 'a2api login' to authenticate")
    print("Install CLI: https://github.com/...")
    sys.exit(1)
```

### Handle Token Revocation

```python
try:
    access_token = refresh_access_token()
except Exception as e:
    print("Your session has expired. Please run 'a2api login' again.")
    sys.exit(1)
```

---

## Next Steps

1. **Install TOML parser** for your language
2. **Read `~/.a2api`** file (see Step 2)
3. **Implement token refresh** logic (see Step 4)
4. **Test with expired token** to verify refresh works
5. **Review security checklist** before shipping
6. **Handle errors gracefully** (guide users to `a2api login`)

---

## Additional Resources

- 📖 [API Documentation](../api.md) - Complete HTTP API reference
- 📖 [OAuth Integration](oauth.md) - For web apps with multiple users
- 📖 [Integration Overview](README.md) - Choose the right integration pattern
- 📖 [CLI Guide](../../.claude/guides/services/cli.md) - How to use the CLI
