# OAuth 2.0 Integration Guide

Complete guide for integrating web applications with a2api using OAuth 2.0.

## Overview

### What is a2api OAuth?

a2api provides OAuth 2.0 authorization to let third-party applications act on behalf of users. When a user authorizes your app, you receive an access token to call the a2api proxy API on their behalf.

### When to Use OAuth

Use OAuth 2.0 when:
- Building a web application (SaaS, internal tool, dashboard)
- Your app needs to proxy API requests for multiple users
- Users should explicitly grant permission to your app
- Your app runs on a server (not a desktop app)

For desktop apps that share the CLI's session, see [Desktop App Integration](desktop-apps.md).

### OAuth Flow Summary

```
User → Your App → a2api Authorization → User Approves →
→ Auth Code → Your App → Token Exchange → Access Token →
→ Call Proxy API
```

---

## Prerequisites

Before you begin:

- **Admin Access:** You need admin privileges to register OAuth clients
- **HTTPS Callback:** Your redirect URI must use HTTPS in production
- **Secure Storage:** Ability to store client secrets securely (server-side only)
- **Web Server:** Backend server to handle OAuth callbacks

---

## Step 1: Register Your OAuth Client

Before users can authorize your app, register it with a2api to receive a `client_id` and `client_secret`.

### API Endpoint

```http
POST https://api.a2api.orchard9.ai/oauth/clients
Content-Type: application/json
Authorization: Bearer <admin-access-token>

{
  "name": "My Analytics Dashboard",
  "redirect_uris": [
    "https://myapp.example.com/auth/callback"
  ]
}
```

### Response

```json
{
  "client_id": "oauth_client_01JH5YQRMGDCK8V7N2W3X4Z5A6",
  "client_secret": "secret_01JH5YQRMGDCK8V7N2W3X4Z5A6B7C8D9E0F1G2H3I4J5K6L7M8N9O0P1Q2R3S4T5U6V7W8X9Y0Z",
  "name": "My Analytics Dashboard",
  "redirect_uris": [
    "https://myapp.example.com/auth/callback"
  ],
  "created_at": "2025-01-15T10:30:00Z"
}
```

### Important Security Notes

⚠️ **Save the `client_secret` immediately - it's only shown once!**

- Store `client_secret` securely (environment variables, secrets manager)
- Never expose `client_secret` in frontend code or version control
- Treat `client_secret` like a password

### cURL Example

```bash
curl -X POST https://api.a2api.orchard9.ai/oauth/clients \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin-token>" \
  -d '{
    "name": "My Analytics Dashboard",
    "redirect_uris": ["https://myapp.example.com/auth/callback"]
  }'
```

---

## Step 2: Authorization Flow

Direct users to a2api's authorization page where they approve/deny your app.

### 2.1 Build Authorization URL

```
https://api.a2api.orchard9.ai/oauth/authorize?client_id=<client_id>&redirect_uri=<redirect_uri>&state=<state>
```

**Query Parameters:**

| Parameter | Required | Description |
|-----------|----------|-------------|
| `client_id` | ✅ Yes | Your OAuth client ID from Step 1 |
| `redirect_uri` | ✅ Yes | Must exactly match a registered redirect URI |
| `state` | ✅ Yes | Random string to prevent CSRF attacks |

### 2.2 Generate State Parameter

The `state` parameter prevents CSRF attacks. Generate a random string and store it in the user's session.

**Example (Node.js):**

```javascript
const crypto = require('crypto');

// Generate random state
const state = crypto.randomBytes(32).toString('hex');

// Store in session
req.session.oauthState = state;

// Build authorization URL
const authUrl = new URL('https://api.a2api.orchard9.ai/oauth/authorize');
authUrl.searchParams.set('client_id', process.env.OAUTH_CLIENT_ID);
authUrl.searchParams.set('redirect_uri', 'https://myapp.example.com/auth/callback');
authUrl.searchParams.set('state', state);

// Redirect user
res.redirect(authUrl.toString());
```

### 2.3 User Approves/Denies

The user sees an authorization page:

```
┌─────────────────────────────────────────┐
│ My Analytics Dashboard                  │
│ wants to access your a2api account      │
│                                         │
│ This will allow the app to:            │
│ • Proxy API requests on your behalf    │
│                                         │
│  [Deny]              [Approve]         │
└─────────────────────────────────────────┘
```

---

## Step 3: Handle Authorization Callback

After the user approves/denies, a2api redirects back to your `redirect_uri`.

### 3.1 Success Callback

If approved, you receive an authorization code:

```
https://myapp.example.com/auth/callback?code=<auth_code>&state=<state>
```

**Query Parameters:**

| Parameter | Description |
|-----------|-------------|
| `code` | Authorization code (single-use, expires in 5 minutes) |
| `state` | The state value you provided in Step 2.2 |

### 3.2 Validate State Parameter

**⚠️ Critical Security Step:** Verify the `state` matches what you stored in Step 2.2.

```javascript
app.get('/auth/callback', (req, res) => {
  const { code, state } = req.query;

  // Validate state to prevent CSRF
  if (state !== req.session.oauthState) {
    return res.status(400).send('Invalid state parameter');
  }

  // Clear state from session
  delete req.session.oauthState;

  // Continue to Step 4...
});
```

### 3.3 Error Callback

If denied or error occurs:

```
https://myapp.example.com/auth/callback?error=access_denied&error_description=User%20denied%20authorization
```

**Error Codes:**

| Error | Description |
|-------|-------------|
| `access_denied` | User denied authorization |
| `invalid_request` | Missing required parameters |
| `unauthorized_client` | Invalid client_id |
| `invalid_redirect_uri` | Redirect URI doesn't match registration |

---

## Step 4: Exchange Code for Tokens

Exchange the authorization code for an access token and refresh token.

### API Endpoint

```http
POST https://api.a2api.orchard9.ai/auth/token/exchange
Content-Type: application/json

{
  "code": "<authorization_code>",
  "client_id": "<your_client_id>",
  "client_secret": "<your_client_secret>",
  "redirect_uri": "<your_redirect_uri>"
}
```

### Response

```json
{
  "access_token": "a2api_access_01JH5YQRMGDCK8V7N2W3X4Z5A6B7C8D9E0F1G2H3I4J5K6",
  "refresh_token": "a2api_refresh_01JH5YQRMGDCK8V7N2W3X4Z5A6B7C8D9E0F1G2H3I4J5K6",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

**Response Fields:**

| Field | Description |
|-------|-------------|
| `access_token` | Use to call proxy API (valid for 1 hour) |
| `refresh_token` | Use to get new access tokens without re-authorization |
| `token_type` | Always "Bearer" |
| `expires_in` | Seconds until access token expires (3600 = 1 hour) |

### Example (Node.js)

```javascript
const axios = require('axios');

app.get('/auth/callback', async (req, res) => {
  const { code, state } = req.query;

  // Validate state (see Step 3.2)
  if (state !== req.session.oauthState) {
    return res.status(400).send('Invalid state parameter');
  }

  try {
    // Exchange code for tokens
    const response = await axios.post('https://api.a2api.orchard9.ai/auth/token/exchange', {
      code,
      client_id: process.env.OAUTH_CLIENT_ID,
      client_secret: process.env.OAUTH_CLIENT_SECRET,
      redirect_uri: 'https://myapp.example.com/auth/callback'
    });

    const { access_token, refresh_token, expires_in } = response.data;

    // Store tokens securely (e.g., encrypted in database)
    await storeUserTokens(req.session.userId, {
      access_token,
      refresh_token,
      expires_at: Date.now() + (expires_in * 1000)
    });

    res.redirect('/dashboard');
  } catch (error) {
    console.error('Token exchange failed:', error.response?.data);
    res.status(500).send('Authorization failed');
  }
});
```

### Security Notes

- Authorization codes are single-use only (attempting reuse returns error)
- Codes expire in 5 minutes
- Never expose tokens in URLs or logs
- Store tokens encrypted in your database

---

## Step 5: Use Access Token with Proxy API

Use the access token to proxy API requests on behalf of the user.

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

### Example (Node.js)

```javascript
async function proxyOpenAIRequest(userId, connectionId, messages) {
  // Get user's access token from database
  const { access_token } = await getUserTokens(userId);

  const response = await axios.post('https://api.a2api.orchard9.ai/proxy', {
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
      'Authorization': `Bearer ${access_token}`
    }
  });

  return response.data;
}
```

---

## Step 6: Refresh Tokens

Access tokens expire after 1 hour. Use the refresh token to get a new access token without re-authorization.

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

⚠️ **a2api uses token family rotation for security:**

1. Each refresh returns a **new** access token + **new** refresh token
2. The old refresh token is invalidated
3. **You must store the new refresh token** to continue refreshing
4. Attempting to reuse an old refresh token **revokes the entire token family**

### Example (Node.js)

```javascript
async function refreshAccessToken(userId) {
  const { refresh_token } = await getUserTokens(userId);

  try {
    const response = await axios.post('https://api.a2api.orchard9.ai/auth/token', {
      grant_type: 'refresh_token',
      refresh_token
    });

    const { access_token, refresh_token: new_refresh_token, expires_in } = response.data;

    // Store NEW tokens (critical for token rotation)
    await storeUserTokens(userId, {
      access_token,
      refresh_token: new_refresh_token,
      expires_at: Date.now() + (expires_in * 1000)
    });

    return access_token;
  } catch (error) {
    if (error.response?.status === 401) {
      // Refresh token expired or revoked - user must re-authorize
      throw new Error('User must re-authorize');
    }
    throw error;
  }
}
```

### Auto-Refresh Pattern

Refresh tokens before they expire to avoid downtime:

```javascript
async function getValidAccessToken(userId) {
  const tokens = await getUserTokens(userId);

  // Refresh if token expires in < 5 minutes
  if (tokens.expires_at < Date.now() + (5 * 60 * 1000)) {
    return await refreshAccessToken(userId);
  }

  return tokens.access_token;
}
```

---

## Security Best Practices

### State Parameter (CSRF Protection)

✅ **DO:**
- Generate cryptographically random state values
- Store state in server-side session
- Validate state matches before accepting auth code
- Clear state after validation

❌ **DON'T:**
- Skip state validation
- Use predictable state values
- Store state in cookies or frontend code

### Redirect URI Validation

✅ **DO:**
- Use exact match validation (not "starts with")
- Register all redirect URIs during client registration
- Use HTTPS in production

❌ **DON'T:**
- Allow wildcard redirect URIs
- Use HTTP in production
- Accept arbitrary redirect URIs

### Client Secret Protection

✅ **DO:**
- Store in environment variables or secrets manager
- Restrict access to backend servers only
- Rotate if compromised

❌ **DON'T:**
- Commit to version control
- Expose in frontend code
- Log in plaintext

### Token Storage

✅ **DO:**
- Encrypt tokens at rest
- Store in secure database (not localStorage)
- Use secure, httpOnly cookies if storing in browser
- Set appropriate token lifetimes

❌ **DON'T:**
- Store in localStorage (XSS vulnerable)
- Log tokens to console/files
- Share tokens between users
- Store in URL parameters

### Token Refresh

✅ **DO:**
- Handle token rotation (store new refresh tokens)
- Refresh before expiry (5-minute buffer)
- Handle revocation gracefully (prompt re-authorization)
- Implement retry logic for transient failures

❌ **DON'T:**
- Reuse old refresh tokens (triggers revocation)
- Wait until token expires (causes downtime)
- Ignore token rotation

---

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `invalid_grant` | Auth code expired or already used | Restart authorization flow |
| `invalid_client` | Wrong client_id or client_secret | Verify credentials |
| `invalid_redirect_uri` | Redirect URI doesn't match registration | Use exact registered URI |
| `access_denied` | User denied authorization | Handle gracefully, explain why needed |
| `token_reuse_detected` | Old refresh token reused | Entire token family revoked, user must re-authorize |

### Token Reuse Detection

If an old refresh token is reused (security attack), a2api revokes the entire token family:

```javascript
try {
  await refreshAccessToken(userId);
} catch (error) {
  if (error.message === 'User must re-authorize') {
    // Token family revoked - prompt user to re-authorize
    return res.redirect('/auth/connect');
  }
}
```

### Handling Revocation

When a user's tokens are revoked:

1. Clear stored tokens from database
2. Redirect user to re-authorization flow
3. Log security event for audit

---

## Complete Example: Express.js Integration

Full working example of OAuth 2.0 integration:

```javascript
const express = require('express');
const session = require('express-session');
const axios = require('axios');
const crypto = require('crypto');

const app = express();

// Session middleware
app.use(session({
  secret: process.env.SESSION_SECRET,
  resave: false,
  saveUninitialized: false,
  cookie: { secure: true, httpOnly: true }
}));

// Environment variables
const OAUTH_CLIENT_ID = process.env.OAUTH_CLIENT_ID;
const OAUTH_CLIENT_SECRET = process.env.OAUTH_CLIENT_SECRET;
const REDIRECT_URI = 'https://myapp.example.com/auth/callback';

// Step 1: Initiate OAuth flow
app.get('/auth/connect', (req, res) => {
  // Generate and store state
  const state = crypto.randomBytes(32).toString('hex');
  req.session.oauthState = state;

  // Build authorization URL
  const authUrl = new URL('https://api.a2api.orchard9.ai/oauth/authorize');
  authUrl.searchParams.set('client_id', OAUTH_CLIENT_ID);
  authUrl.searchParams.set('redirect_uri', REDIRECT_URI);
  authUrl.searchParams.set('state', state);

  res.redirect(authUrl.toString());
});

// Step 2: Handle callback
app.get('/auth/callback', async (req, res) => {
  const { code, state, error } = req.query;

  // Handle user denial
  if (error) {
    return res.status(400).send(`Authorization failed: ${error}`);
  }

  // Validate state
  if (state !== req.session.oauthState) {
    return res.status(400).send('Invalid state parameter');
  }
  delete req.session.oauthState;

  try {
    // Exchange code for tokens
    const tokenResponse = await axios.post(
      'https://api.a2api.orchard9.ai/auth/token/exchange',
      {
        code,
        client_id: OAUTH_CLIENT_ID,
        client_secret: OAUTH_CLIENT_SECRET,
        redirect_uri: REDIRECT_URI
      }
    );

    const { access_token, refresh_token, expires_in } = tokenResponse.data;

    // Store tokens (example: in session, use database in production)
    req.session.tokens = {
      access_token,
      refresh_token,
      expires_at: Date.now() + (expires_in * 1000)
    };

    res.redirect('/dashboard');
  } catch (error) {
    console.error('Token exchange failed:', error.response?.data);
    res.status(500).send('Authorization failed');
  }
});

// Helper: Get valid access token (with auto-refresh)
async function getValidAccessToken(session) {
  const tokens = session.tokens;

  // Refresh if expiring soon
  if (tokens.expires_at < Date.now() + (5 * 60 * 1000)) {
    const response = await axios.post('https://api.a2api.orchard9.ai/auth/token', {
      grant_type: 'refresh_token',
      refresh_token: tokens.refresh_token
    });

    // Update stored tokens
    session.tokens = {
      access_token: response.data.access_token,
      refresh_token: response.data.refresh_token,
      expires_at: Date.now() + (response.data.expires_in * 1000)
    };

    return response.data.access_token;
  }

  return tokens.access_token;
}

// Step 3: Use access token
app.post('/api/proxy', async (req, res) => {
  try {
    const access_token = await getValidAccessToken(req.session);

    const response = await axios.post(
      'https://api.a2api.orchard9.ai/proxy',
      {
        connection_id: req.body.connection_id,
        request: req.body.request
      },
      {
        headers: {
          'Authorization': `Bearer ${access_token}`
        }
      }
    );

    res.json(response.data);
  } catch (error) {
    if (error.response?.status === 401) {
      // Token revoked - prompt re-authorization
      return res.status(401).json({ error: 'Please re-authorize' });
    }
    res.status(500).json({ error: 'Proxy request failed' });
  }
});

app.listen(3000);
```

---

## Testing

### Test Authorization Flow

1. Visit `/auth/connect` in browser
2. Verify redirect to a2api authorization page
3. Approve application
4. Verify redirect back to `/auth/callback` with code
5. Check tokens stored correctly

### Test Token Refresh

```javascript
// Manually expire token to test refresh
req.session.tokens.expires_at = Date.now() - 1000;

// Make proxy request - should auto-refresh
const response = await axios.post('/api/proxy', { ... });
```

### Test Revocation

```javascript
// Attempt to reuse old refresh token
const oldRefreshToken = req.session.tokens.refresh_token;

// Refresh once (gets new tokens)
await axios.post('/auth/token', {
  grant_type: 'refresh_token',
  refresh_token: oldRefreshToken
});

// Try to reuse old token - should fail
await axios.post('/auth/token', {
  grant_type: 'refresh_token',
  refresh_token: oldRefreshToken
}); // 401 Unauthorized - token family revoked
```

---

## Next Steps

1. **Register your OAuth client** using Step 1
2. **Implement authorization flow** using the Express.js example
3. **Test the complete flow** end-to-end
4. **Review security checklist** before production deployment
5. **Implement proper token storage** (encrypted database)
6. **Set up monitoring** for token refresh failures
7. **Document for your users** how to authorize your app

---

## Additional Resources

- 📖 [API Documentation](../api.md) - Complete HTTP API reference
- 📖 [Desktop App Integration](desktop-apps.md) - For desktop apps sharing CLI credentials
- 📖 [Integration Overview](README.md) - Choose the right integration pattern
- 🔍 [OAuth Spec](../specs/web-app-sso.md) - Formal OAuth implementation specification
