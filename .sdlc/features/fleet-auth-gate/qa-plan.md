# QA Plan: fleet-auth-gate

## QA1: Middleware manifest renders correctly

**Method:** Helm template rendering
**Steps:**
1. Run `helm template` with `auth.enabled=true` and `auth.proxyNamespace=sdlc-hub`
2. Verify the rendered `Middleware` CRD has kind `Middleware`, apiVersion `traefik.io/v1alpha1`
3. Verify `forwardAuth.address` points to `http://oauth2-proxy.sdlc-hub.svc.cluster.local:4180/oauth2/auth`
4. Verify `authResponseHeaders` includes `X-Auth-Request-Email` and `X-Auth-Request-User`
5. Verify `trustForwardHeader: true` is set

**Pass:** All fields render as specified.

## QA2: Middleware is omitted when auth.enabled is false

**Method:** Helm template rendering
**Steps:**
1. Run `helm template` with `auth.enabled=false` (the default)
2. Verify no `Middleware` CRD is rendered
3. Verify the `IngressRoute` has no `middlewares` block

**Pass:** No auth resources rendered when disabled.

## QA3: IngressRoute renders with middleware when auth enabled

**Method:** Helm template rendering
**Steps:**
1. Run `helm template` with `auth.enabled=true`, `project.slug=myproject`, `ingress.domain=sdlc.threesix.ai`
2. Verify `IngressRoute` matches host `myproject.sdlc.threesix.ai`
3. Verify `middlewares` section includes `name: sdlc-google-auth`
4. Verify TLS secret name matches `ingress.tlsSecretName`

**Pass:** IngressRoute correctly wires the middleware.

## QA4: IngressRoute renders without middleware when auth disabled

**Method:** Helm template rendering
**Steps:**
1. Run `helm template` with `auth.enabled=false`
2. Verify `IngressRoute` renders but has no `middlewares` section

**Pass:** Clean IngressRoute without auth when disabled.

## QA5: Unauthenticated request redirects to Google sign-in

**Method:** Manual curl against live cluster (post-deployment)
**Steps:**
1. `curl -v https://sdlc-sdlc.threesix.ai/ 2>&1`
2. Verify response is 302 with `Location` header pointing to Google accounts sign-in URL

**Pass:** 302 redirect to Google OAuth.

## QA6: Authenticated request passes through

**Method:** Manual curl with valid cookie (post-deployment)
**Steps:**
1. Complete Google sign-in in a browser
2. Extract `_oauth2_proxy` cookie value
3. `curl -v -H "Cookie: _oauth2_proxy=<value>" https://sdlc-sdlc.threesix.ai/`
4. Verify 200 response with sdlc UI content

**Pass:** 200 with valid session cookie.

## QA7: Bearer token bypasses Google OAuth

**Method:** Manual curl with bearer token (post-deployment)
**Steps:**
1. `curl -v -H "Authorization: Bearer <valid-sdlc-token>" https://sdlc-sdlc.threesix.ai/api/state`
2. Verify 200 JSON response

**Pass:** Bearer token auth works without Google session.

## QA8: Cross-subdomain SSO works

**Method:** Manual browser test (post-deployment)
**Steps:**
1. Sign in at `sdlc.threesix.ai` via Google
2. Navigate to `sdlc-sdlc.threesix.ai` without signing in again
3. Verify the page loads without a second Google redirect

**Pass:** Single sign-in covers all subdomains.

## QA9: values.yaml defaults are safe

**Method:** Inspection
**Steps:**
1. Read `values.yaml` and verify `auth.enabled` defaults to `false`
2. Verify `auth.proxyNamespace` defaults to `sdlc-hub`
3. Confirm existing deployments that don't set auth values are unaffected

**Pass:** Defaults preserve backward compatibility.
