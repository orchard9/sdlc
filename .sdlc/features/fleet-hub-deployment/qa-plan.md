# QA Plan: Deploy hub mode sdlc-server at sdlc.threesix.ai

## Test Strategy

This is an infrastructure deployment feature. QA focuses on verifying the deployed resources are correct, the auth flow works, and the hub API is accessible. All tests are manual verification steps against the live cluster.

## Pre-deployment Checks

### Q1: Manifest Lint
- All YAML files in `k3s-fleet/deployments/hub/` are valid YAML
- `kubectl apply --dry-run=client -f k3s-fleet/deployments/hub/` succeeds without errors
- No hardcoded secrets in committed manifests

### Q2: Namespace Isolation
- Hub resources are in `sdlc-hub` namespace, not in any `sdlc-*` project namespace
- No resource naming conflicts with existing deployments

## Deployment Verification

### Q3: Pod Health
- `kubectl get pods -n sdlc-hub` shows two pods: `sdlc-hub-*` and `oauth2-proxy-*`
- Both pods reach `Running` state with `1/1` ready containers
- `kubectl logs -n sdlc-hub deployment/sdlc-hub` shows "SDLC hub server started"
- `kubectl logs -n sdlc-hub deployment/oauth2-proxy` shows no error-level messages

### Q4: Service Connectivity
- `kubectl get svc -n sdlc-hub` shows `sdlc-hub` and `oauth2-proxy` services
- From within the cluster: `curl http://sdlc-hub.sdlc-hub.svc.cluster.local/api/health` returns 200
- From within the cluster: `curl http://oauth2-proxy.sdlc-hub.svc.cluster.local/ping` returns 200

### Q5: IngressRoute and Middleware
- `kubectl get ingressroute -n sdlc-hub` shows the hub IngressRoute
- `kubectl get middleware -n sdlc-hub` shows `sdlc-hub-auth`
- Traefik dashboard shows the route is active (no errors)

## Authentication Flow

### Q6: Unauthenticated Access Redirects
- `curl -sI https://sdlc.threesix.ai/` returns HTTP 302 with `Location` containing `accounts.google.com`
- `curl -sI https://sdlc.threesix.ai/api/hub/projects` returns HTTP 302 (not 200)

### Q7: Google OAuth Login
- Navigate to `https://sdlc.threesix.ai` in browser
- Redirected to Google sign-in
- Sign in with an allowed-domain email (livelyideo.tv, masq.me, or virtualcommunities.ai)
- Successfully redirected back to the hub UI
- Cookie `_sdlc_hub_oauth2` is set in the browser

### Q8: Unauthorized Domain Rejection
- Attempt sign-in with an email outside allowed domains
- oauth2-proxy returns 403 Forbidden

## Hub API Verification

### Q9: Hub Projects Endpoint
- After auth, `GET https://sdlc.threesix.ai/api/hub/projects` returns 200 with JSON array
- Response is valid JSON (empty array `[]` if no projects have heartbeated)

### Q10: Hub SSE Events
- After auth, `curl -N https://sdlc.threesix.ai/api/hub/events` opens an SSE stream
- Stream sends keepalive comments (`:` lines)

### Q11: Health Endpoint
- `GET https://sdlc.threesix.ai/api/health` returns 200
- Health check is accessible via both direct pod access and through ingress

## TLS Verification

### Q12: Certificate Valid
- `curl -vI https://sdlc.threesix.ai 2>&1 | grep 'subject:'` shows `*.sdlc.threesix.ai`
- Certificate is issued by Let's Encrypt
- No browser TLS warnings when navigating to the site

## DNS Verification

### Q13: DNS Resolution
- `dig sdlc.threesix.ai` returns the correct cluster ingress IP
- Response matches the IP used by `*.sdlc.threesix.ai` wildcard

## Pass Criteria

All Q1–Q13 checks pass. The hub is reachable, authenticated, and serving the hub API.
