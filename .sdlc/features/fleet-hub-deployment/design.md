# Design: Deploy hub mode sdlc-server at sdlc.threesix.ai

## Overview

This is a pure infrastructure/deployment feature — no application code changes required. The hub mode already exists in `sdlc-server`. This design covers the Kubernetes manifests, their relationships, and the deployment procedure.

## Manifest Design

### 1. Namespace (`namespace.yaml`)

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: sdlc-hub
  labels:
    app.kubernetes.io/managed-by: sdlc-fleet
    app.kubernetes.io/part-of: sdlc-hub
```

### 2. sdlc-hub Deployment (`sdlc-hub-deployment.yaml`)

Single-container deployment running the hub-mode server. No git-sync sidecar — the hub does not serve a project workspace.

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sdlc-hub
  namespace: sdlc-hub
spec:
  replicas: 1
  selector:
    matchLabels:
      app: sdlc-hub
  template:
    spec:
      containers:
        - name: sdlc-server
          image: ghcr.io/orchard9/sdlc:latest
          args: ["serve", "--hub", "--port", "8080"]
          ports:
            - containerPort: 8080
          env:
            - name: SDLC_HUB
              value: "true"
            - name: SDLC_ROOT
              value: /tmp/sdlc-hub
          resources:
            requests: { cpu: 50m, memory: 64Mi }
            limits:   { cpu: 250m, memory: 128Mi }
          livenessProbe:
            httpGet:
              path: /api/health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 15
          readinessProbe:
            httpGet:
              path: /api/health
              port: 8080
            initialDelaySeconds: 3
            periodSeconds: 10
```

Key decisions:
- `SDLC_ROOT=/tmp/sdlc-hub` — hub mode needs a root dir but does not use it for project state; `/tmp` is fine since hub state persists to `~/.sdlc/hub-state.yaml`
- Lower resource limits than per-project instances — no agent runs, just registry + SSE
- Both `--hub` flag and `SDLC_HUB=true` env for clarity (either works)

### 3. oauth2-proxy Deployment (`oauth2-proxy-deployment.yaml`)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: oauth2-proxy
  namespace: sdlc-hub
spec:
  replicas: 1
  selector:
    matchLabels:
      app: oauth2-proxy
  template:
    spec:
      containers:
        - name: oauth2-proxy
          image: quay.io/oauth2-proxy/oauth2-proxy:v7.6.0
          args:
            - --provider=google
            - --email-domain=livelyideo.tv
            - --email-domain=masq.me
            - --email-domain=virtualcommunities.ai
            - --upstream=static://200
            - --http-address=0.0.0.0:4180
            - --redirect-url=https://sdlc.threesix.ai/oauth2/callback
            - --cookie-secure=true
            - --cookie-name=_sdlc_hub_oauth2
            - --set-xauthrequest=true
            - --reverse-proxy=true
            - --pass-host-header=true
          env:
            - name: OAUTH2_PROXY_CLIENT_ID
              valueFrom:
                secretKeyRef:
                  name: oauth2-proxy-credentials
                  key: client-id
            - name: OAUTH2_PROXY_CLIENT_SECRET
              valueFrom:
                secretKeyRef:
                  name: oauth2-proxy-credentials
                  key: client-secret
            - name: OAUTH2_PROXY_COOKIE_SECRET
              valueFrom:
                secretKeyRef:
                  name: oauth2-proxy-credentials
                  key: cookie-secret
          ports:
            - containerPort: 4180
          resources:
            requests: { cpu: 10m, memory: 32Mi }
            limits:   { cpu: 100m, memory: 64Mi }
          livenessProbe:
            httpGet:
              path: /ping
              port: 4180
            periodSeconds: 15
```

### 4. Services

**sdlc-hub-service.yaml:**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: sdlc-hub
  namespace: sdlc-hub
spec:
  selector:
    app: sdlc-hub
  ports:
    - port: 80
      targetPort: 8080
```

**oauth2-proxy-service.yaml:**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: oauth2-proxy
  namespace: sdlc-hub
spec:
  selector:
    app: oauth2-proxy
  ports:
    - port: 80
      targetPort: 4180
```

### 5. Traefik ForwardAuth Middleware (`middleware-forward-auth.yaml`)

```yaml
apiVersion: traefik.io/v1alpha1
kind: Middleware
metadata:
  name: sdlc-hub-auth
  namespace: sdlc-hub
spec:
  forwardAuth:
    address: http://oauth2-proxy.sdlc-hub.svc.cluster.local/oauth2/auth
    trustForwardHeader: true
    authResponseHeaders:
      - X-Auth-Request-User
      - X-Auth-Request-Email
```

This middleware delegates authentication to oauth2-proxy. Traefik sends a subrequest to the `/oauth2/auth` endpoint; if the user has a valid cookie, oauth2-proxy returns 200 and the request proceeds. If not, it returns 401 and Traefik redirects the user to the sign-in page.

### 6. IngressRoute (`ingressroute.yaml`)

Two IngressRoute resources:

**Main route (authenticated):**
```yaml
apiVersion: traefik.io/v1alpha1
kind: IngressRoute
metadata:
  name: sdlc-hub
  namespace: sdlc-hub
spec:
  entryPoints:
    - websecure
  routes:
    - match: Host(`sdlc.threesix.ai`) && !PathPrefix(`/oauth2`)
      kind: Rule
      middlewares:
        - name: sdlc-hub-auth
      services:
        - name: sdlc-hub
          port: 80
    - match: Host(`sdlc.threesix.ai`) && PathPrefix(`/oauth2`)
      kind: Rule
      services:
        - name: oauth2-proxy
          port: 80
  tls:
    secretName: sdlc-wildcard-tls
```

The `/oauth2/*` paths (callback, sign-in, sign-out) go directly to oauth2-proxy without the forward-auth middleware — otherwise the auth check would loop.

## Request Flow

```
1. Browser → GET https://sdlc.threesix.ai/
2. Traefik matches IngressRoute → applies sdlc-hub-auth middleware
3. Middleware → GET http://oauth2-proxy.sdlc-hub/oauth2/auth
4a. No cookie → 401 → Traefik returns 302 to /oauth2/start?rd=/
4b. Valid cookie → 200 + X-Auth-Request-Email header
5. Request forwarded to sdlc-hub:80 → sdlc-server serves hub UI
```

## Health Check Bypass

The `/api/health` endpoint should remain accessible without auth for k8s probes. Since the liveness/readiness probes hit the container directly (not through the IngressRoute), they bypass oauth2-proxy naturally. No special routing needed.

## DNS Setup

Use Cloudflare API to create/update an A record:
- Name: `sdlc` (under `threesix.ai` zone)
- Content: same IP as `*.sdlc.threesix.ai` wildcard
- Proxied: false (direct to cluster, TLS via cert-manager)

## Deployment Order

1. Create namespace `sdlc-hub`
2. Create Secret `oauth2-proxy-credentials` with GCP OAuth client ID/secret and cookie secret
3. Apply oauth2-proxy deployment + service
4. Apply sdlc-hub deployment + service
5. Apply middleware + IngressRoute
6. Verify: `curl -I https://sdlc.threesix.ai` → 302 to Google

## File Layout

```
k3s-fleet/deployments/hub/
├── namespace.yaml                   # sdlc-hub namespace
├── sdlc-hub-deployment.yaml         # Hub-mode sdlc-server
├── sdlc-hub-service.yaml            # ClusterIP for sdlc-hub
├── oauth2-proxy-deployment.yaml     # Google OAuth proxy
├── oauth2-proxy-service.yaml        # ClusterIP for oauth2-proxy  
├── middleware-forward-auth.yaml     # Traefik ForwardAuth → oauth2-proxy
└── ingressroute.yaml                # Traefik IngressRoute for sdlc.threesix.ai
```
