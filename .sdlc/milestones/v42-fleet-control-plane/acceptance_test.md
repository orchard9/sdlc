# Acceptance Test: v42-fleet-control-plane

## Scenario: Authenticated fleet management end-to-end

### Prerequisites
- Cluster is running with at least one sdlc instance deployed
- GCP OAuth client configured for `sdlc.threesix.ai`
- Hub mode sdlc-server deployed in `sdlc-hub` namespace with native OAuth2
- Test user has a Google account on an allowed domain

### Steps

1. **Auth gate works**
   - Navigate to `https://sdlc.threesix.ai` in a browser
   - Verify: redirected to Google sign-in (not the hub page)
   - Sign in with an allowed Google account
   - Verify: redirected back to the hub dashboard

2. **Auth gate blocks unauthorized**
   - Open an incognito window
   - Navigate to `https://sdlc.sdlc.threesix.ai`
   - Verify: redirected to Google sign-in (individual instances are also gated via Traefik forwardAuth)

3. **Machine-to-machine auth works**
   - Call `GET /api/hub/fleet` with a valid service token in Authorization header
   - Verify: returns fleet data without browser session

4. **Running instances visible**
   - On the hub dashboard, verify at least `sdlc-sdlc` appears as a running instance
   - Verify: shows health status (green dot), clickable URL
   - Verify: `sdlc-tls` and other non-instance namespaces do NOT appear
   - Click the instance card
   - Verify: navigates to `https://sdlc.sdlc.threesix.ai` (authenticated, no re-login — cookie on `.sdlc.threesix.ai`)

5. **Search works at scale**
   - Type a project name in the search bar (should be autofocused)
   - Verify: both running and available lists filter instantly
   - Verify: search is the primary interaction, not buried in a sidebar

6. **Fleet-wide agent status visible**
   - Verify: the dashboard header shows aggregate count of active agent runs
     across all instances (e.g., "3 agents running across 2 projects")

7. **Available repos visible**
   - Verify there is an "Available" section showing repos from the Gitea org
     that don't have running instances
   - Verify: at least one repo appears with a "Start" action
   - Verify: a brief explanation of what "Start" does is visible for first-time users

8. **Start an instance**
   - Click "Start" on an available repo
   - Verify: status changes to "provisioning"
   - Wait for provisioning to complete (SSE update)
   - Verify: the repo moves from "Available" to "Running" with a healthy status

9. **Import an external repo**
   - Click "Import" or navigate to the import section
   - Paste a public GitHub repo URL
   - Click import
   - Verify: repo appears in Gitea org (check `https://git.threesix.ai/orchard9/<name>`)
   - Verify: instance is automatically provisioned and appears in running list

10. **Session management**
    - Click logout (or POST /auth/logout)
    - Verify: session cookie cleared, accessing any sdlc URL requires re-login

### Pass criteria
All 10 steps pass. The fleet control plane is the single authenticated entry point
for all sdlc infrastructure — native OAuth2 (no oauth2-proxy), signed session cookies
on `.sdlc.threesix.ai` for SSO across all instances.
