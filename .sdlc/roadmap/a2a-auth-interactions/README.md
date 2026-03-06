# Integration Guide

Choose how to integrate your application with a2api based on your use case.

## Which Integration Pattern?

```
┌─────────────────────────────────────────────────────────────┐
│ What type of application are you building?                  │
└─────────────────────────────────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
        ▼                 ▼                 ▼
   Web App         Desktop App        CLI/Script
   (SaaS, etc)     (Electron, etc)    (Automation)
        │                 │                 │
        ▼                 ▼                 ▼
  OAuth 2.0        CLI Credentials     Direct API
```

## Comparison Table

| Pattern | Best For | Auth Flow | Complexity | Multi-User |
|---------|----------|-----------|------------|------------|
| **OAuth 2.0** | Web apps, SaaS platforms | User authorizes your app | Medium | ✅ Yes |
| **CLI Credentials** | Desktop apps on same machine | Read ~/.a2api file | Low | ❌ No (single user) |
| **Direct API** | Scripts, automation | Use CLI login | Very Low | ❌ No |

## Integration Patterns

### 1. OAuth 2.0 (Web Apps)

**Use when:**
- Building a web application (SaaS, internal tool, dashboard)
- Need to act on behalf of multiple users
- Users grant permission to your app
- Your app runs on a server

**How it works:**
1. Register your OAuth client with a2api
2. Redirect users to a2api authorization page
3. User approves your app
4. Receive authorization code
5. Exchange code for access token
6. Use token to call proxy API

**Get Started:**
- 📖 [OAuth Integration Guide](oauth.md) - Complete implementation guide
- 🚀 [OAuth Quickstart](../quickstarts/web-app-oauth.md) - 5-minute example

**Example Use Case:**
> "I'm building an analytics dashboard that needs to proxy OpenAI requests for my users. Each user authorizes my app to access their a2api account."

---

### 2. CLI Credentials (Desktop Apps)

**Use when:**
- Building a desktop application (Electron, Tauri, native)
- App runs on same machine as a2api CLI
- Single user per machine
- Want to share CLI's existing session

**How it works:**
1. User runs `a2api login` (one-time setup)
2. Your app reads `~/.a2api` credentials file
3. Use access token to call proxy API
4. Refresh token when expired

**Get Started:**
- 📖 [Desktop App Integration Guide](desktop-apps.md) - Complete implementation guide
- 🚀 [Desktop App Quickstart](../quickstarts/desktop-app.md) - 5-minute example

**Example Use Case:**
> "I'm building an Electron app that co-exists with the a2api CLI. Instead of making users log in twice, I read the CLI's credentials."

---

### 3. Direct API (CLI/Scripts)

**Use when:**
- Writing shell scripts or automation
- Using a2api CLI directly
- Simple one-off tasks

**How it works:**
1. User runs `a2api login`
2. CLI handles all auth automatically
3. Use `a2api proxy` command

**Get Started:**
- 📖 [API Documentation](../api.md) - HTTP API reference
- 📖 [CLI Guide](../../.claude/guides/services/cli.md) - CLI usage

**Example Use Case:**
> "I have a shell script that needs to call OpenAI. I use `a2api proxy` to route requests through the gateway."

---

## Prerequisites

### All Integration Types
- Access to a running a2api gateway
- Basic understanding of HTTP APIs
- HTTPS for production (required for OAuth)

### OAuth 2.0 Specific
- Admin access to register OAuth clients
- Web server to handle OAuth callbacks
- Ability to securely store client secrets

### CLI Credentials Specific
- a2api CLI installed on user's machine
- User has run `a2api login` at least once
- App runs on same machine as CLI

---

## Security Considerations

### OAuth 2.0
- ✅ **DO:** Use state parameter to prevent CSRF
- ✅ **DO:** Validate redirect URIs exactly
- ✅ **DO:** Store client secrets securely (never in frontend code)
- ✅ **DO:** Use HTTPS for all OAuth flows
- ❌ **DON'T:** Expose client secrets in client-side code
- ❌ **DON'T:** Skip state validation

### CLI Credentials
- ✅ **DO:** Verify `~/.a2api` has 0600 permissions
- ✅ **DO:** Refresh tokens before expiry
- ✅ **DO:** Handle token rotation (store new tokens)
- ❌ **DON'T:** Log tokens to console/files
- ❌ **DON'T:** Share credentials between machines

### All Patterns
- ✅ **DO:** Use TLS/HTTPS in production
- ✅ **DO:** Handle token expiry gracefully
- ✅ **DO:** Implement proper error handling
- ❌ **DON'T:** Store tokens in source control
- ❌ **DON'T:** Log sensitive credentials

---

## Next Steps

1. **Choose your integration pattern** from the comparison table above
2. **Read the quickstart guide** to get up and running in 5 minutes
3. **Read the full integration guide** for production implementation
4. **Test your integration** against the a2api gateway
5. **Review security checklist** before deploying to production

---

## Support

- 🐛 **Found a bug?** Report at https://github.com/anthropics/claude-code/issues
- 📖 **More docs:** See [CLAUDE.md](../../CLAUDE.md) for complete documentation index
- 🔍 **API Reference:** See [docs/api.md](../api.md) for HTTP API details
