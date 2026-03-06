# Tasks: fleet-auth-gate

## T1: Create shared Traefik middleware sdlc-google-auth pointing to hub oauth2-proxy

Create `k3s-fleet/deployments/helm/sdlc-server/templates/middleware-google-auth.yaml` — a Traefik `Middleware` CRD of type `forwardAuth` that sends auth subrequests to `oauth2-proxy.sdlc-hub.svc.cluster.local:4180/oauth2/auth`. Gated by `auth.enabled` value. Passes `X-Auth-Request-Email` and `X-Auth-Request-User` response headers.

## T2: Apply middleware to v18 fleet-reconcile Helm template so new project ingresses auto-get auth

Create `k3s-fleet/deployments/helm/sdlc-server/templates/ingressroute.yaml` — a Traefik `IngressRoute` CRD that routes `<slug>.<domain>` to the sdlc-server service. When `auth.enabled` is true, the route includes the `sdlc-google-auth` middleware. Add `auth.enabled` (default false) and `auth.proxyNamespace` (default `sdlc-hub`) to `values.yaml`.

## T3: Retrofit existing sdlc-sdlc ingress to use the shared middleware

Update the existing sdlc-sdlc deployment to use the new Helm chart with `auth.enabled: true`. This may mean redeploying sdlc-sdlc through the fleet-reconcile pipeline with the updated chart, or manually applying the middleware annotation to the existing IngressRoute.

## T4: Test: unauthenticated request to sdlc-sdlc.threesix.ai redirects to Google sign-in

Verify with curl that an unauthenticated GET to `https://sdlc-sdlc.threesix.ai` returns a 302 redirect to Google sign-in. Verify that after completing Google auth, the request succeeds with a 200. This is a manual verification step — document the curl commands and expected responses.

## T5: [user-gap] Session expiry and logout — configure oauth2-proxy cookie TTL and expose logout endpoint

Document the oauth2-proxy configuration needed in `fleet-hub-deployment`: cookie domain `.sdlc.threesix.ai`, TTL 24h, refresh 1h, `SameSite=Lax`. Ensure the logout endpoint `/oauth2/sign_out` is accessible. This configuration is owned by `fleet-hub-deployment` — this task captures the requirements and creates a coordination note.

## T6: [user-gap] Service token path — bearer token auth that bypasses oauth2-proxy for machine-to-machine API calls

Configure oauth2-proxy with `--pass-authorization-header=true` so bearer tokens are forwarded to sdlc-server. The existing `auth.rs` bearer validation handles the rest. Document this in the design as the recommended approach. No code changes needed in sdlc-server.
