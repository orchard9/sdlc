# QA Results: fleet-auth-gate

## QA1: Middleware manifest renders correctly — PASS

Ran `helm template` with `auth.enabled=true`. Rendered Middleware CRD has:
- `kind: Middleware`, `apiVersion: traefik.io/v1alpha1`
- `forwardAuth.address: http://oauth2-proxy.sdlc-hub.svc.cluster.local/oauth2/auth`
- `authResponseHeaders: [X-Auth-Request-Email, X-Auth-Request-User]`
- `trustForwardHeader: true`

## QA2: Middleware is omitted when auth.enabled is false — PASS

Ran `helm template` with default values (`auth.enabled=false`). Output contains no `Middleware` CRD. The `IngressRoute` has no `middlewares` block.

## QA3: IngressRoute renders with middleware when auth enabled — PASS

With `auth.enabled=true`, `project.slug=myproject`, `ingress.domain=sdlc.threesix.ai`:
- Route matches `Host(\`myproject.sdlc.threesix.ai\`)`
- `middlewares` includes `name: sdlc-google-auth`
- TLS secret is `sdlc-wildcard-tls`

## QA4: IngressRoute renders without middleware when auth disabled — PASS

With `auth.enabled=false`:
- IngressRoute renders with the correct host match
- No `middlewares` section present

## QA5: Unauthenticated request redirects to Google sign-in — DEFERRED

Requires live cluster with oauth2-proxy deployed. Will be verified during milestone UAT.

## QA6: Authenticated request passes through — DEFERRED

Requires live cluster with valid Google session. Will be verified during milestone UAT.

## QA7: Bearer token bypasses Google OAuth — DEFERRED

Requires live cluster. `--pass-authorization-header=true` is configured in oauth2-proxy deployment. Will be verified during milestone UAT.

## QA8: Cross-subdomain SSO works — DEFERRED

Requires live cluster. `--cookie-domain=.sdlc.threesix.ai` and `--whitelist-domain=.sdlc.threesix.ai` are configured. Will be verified during milestone UAT.

## QA9: values.yaml defaults are safe — PASS

- `auth.enabled` defaults to `false`
- `auth.proxyNamespace` defaults to `sdlc-hub`
- Helm template with no auth overrides produces no auth resources — backward compatible

## Summary

- 5/9 tests PASS (template rendering and defaults)
- 4/9 tests DEFERRED (require live cluster with oauth2-proxy — covered by milestone v42-fleet-control-plane UAT)
- 0 tests FAIL
