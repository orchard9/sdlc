# Tasks: fleet-deploy-hub

## Execution Order

1. **[T1] GCP OAuth client** — already done by user
2. **[T3] DNS** — Cloudflare A record sdlc.threesix.ai
3. **[T4] Namespace** — kubectl create ns sdlc-hub + copy TLS secret
4. **[T2] OAuth secret** — k8s Secret with client ID, secret, session secret
5. **[T8] Fleet tokens** — k8s Secret with Gitea + Woodpecker tokens
6. **[T9] RBAC** — ServiceAccount + ClusterRole
7. **[T5] Helm update** — middleware-google-auth.yaml already points to hub
8. **[T6] IngressRoute** — for sdlc.threesix.ai
9. **[T4] Deploy** — sdlc-hub deployment + service
10. **[T7] Enable auth** — helm upgrade sdlc-sdlc with auth.enabled=true
11. **[T10] Smoke test** — curl + browser verification
