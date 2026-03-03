# Security Audit: Wildcard TLS cert for *.sdlc.threesix.ai via cert-manager DNS01

## Audit Scope

This feature adds wildcard TLS certificate management for the fleet. The security surface covers:
- Certificate storage and access controls
- DNS01 challenge credential handling
- Cross-namespace secret replication scope
- TLS configuration quality

## Findings

### A1 — Wildcard cert scope is broad: all *.sdlc.threesix.ai subdomains — ACCEPTED

**Severity:** Low
**Finding:** The wildcard covers all subdomains of `sdlc.threesix.ai`. Any service with access to the `sdlc-wildcard-tls` Secret in a `sdlc-*` namespace can present a valid TLS certificate for any subdomain.
**Analysis:** This is the intended design. Each subdomain maps to exactly one project namespace and one authorized sdlc-server deployment. No user-controlled subdomains exist. The wildcard is tighter than a full `*.threesix.ai` wildcard (which already exists in the cluster for other services). The `sdlc.*` prefix provides clear scope separation.
**Action:** ACCEPTED. Documented.

---

### A2 — TLS private key stored in Kubernetes Secret — ACCEPTED WITH MONITORING

**Severity:** Medium
**Finding:** The `sdlc-wildcard-tls` Secret containing the private key is stored as a Kubernetes Secret. These are base64-encoded, not encrypted at rest by default in k3s.
**Analysis:** The k3s cluster is a private, Tailscale-gated cluster. The ETCD (or SQLite) datastore is on a controlled node. Secrets are not exposed to public networks. This is the standard cert-manager operational model and is acceptable for this deployment context.
**Mitigations in place:**
- RBAC: Only namespaces that need the secret have access (via reflector which copies to `sdlc-*` namespaces, not all namespaces)
- The `sdlc-tls` source namespace is isolated (no application workloads run there)
**Action:** ACCEPTED. Recommend enabling etcd encryption at rest as a separate cluster-hardening task if not already enabled.

---

### A3 — Cross-namespace replication via reflector limited to sdlc-.* pattern — PASS

**Severity:** Low
**Finding:** The reflector annotations use `sdlc-.*` as the namespace pattern. This limits replication to namespaces whose names start with `sdlc-`, not all namespaces.
**Analysis:** Correct scoping. Only fleet project namespaces match `sdlc-<slug>`. System namespaces (`kube-system`, `cert-manager`, `default`, etc.) do not match.
**Action:** PASS. No change needed.

---

### A4 — Cloudflare API token is cluster-wide, not scoped to sdlc.threesix.ai zone only — ACCEPTED

**Severity:** Medium
**Finding:** The `cloudflare-api-token` Secret in `cert-manager` is shared across all ClusterIssuer solvers for `threesix.ai`, `masq-ops.orchard9.ai`, etc. If the token is compromised, an attacker could modify DNS for all zones it covers.
**Analysis:** This is a pre-existing condition shared with all other cert-manager issuance in the cluster. This feature did not create or change the token. Recommendation is to use a zone-scoped Cloudflare API token with `Zone:DNS:Edit` only for `threesix.ai`, but this is a cluster-wide improvement, not specific to this feature.
**Action:** ACCEPTED for this feature. Track as cluster-hardening improvement: `sdlc task add fleet-ingress-tls "Replace shared cloudflare-api-token with zone-scoped token for threesix.ai"` — see T8 below.

---

### A5 — Certificate uses Let's Encrypt production with correct ACME server — PASS

**Severity:** N/A
**Finding:** `https://acme-v02.api.letsencrypt.org/directory` (production, not staging).
**Analysis:** Correct. Certificate is browser-trusted.
**Action:** PASS.

---

### A6 — No HTTP→HTTPS redirect enforcement by this feature — NOTED

**Severity:** Low
**Finding:** This feature creates the TLS certificate but does not configure HTTP→HTTPS redirect rules. Ingresses in the fleet could potentially serve HTTP.
**Analysis:** HTTP→HTTPS redirect is an Ingress-level concern handled by the fleet Helm chart (Traefik annotation). This feature's scope is certificate issuance only.
**Action:** NOTED. Helm chart spec (fleet-helm-chart) should include `traefik.ingress.kubernetes.io/redirect-entry-point: https` annotation — deferred to that feature.

---

### A7 — Certificate renewal is automatic and tested — PASS

**Severity:** N/A
**Finding:** `renewBefore: 720h` (30 days) ensures renewal well before expiry.
**Analysis:** cert-manager polls every 8 hours and will trigger renewal at the renewal time (2026-05-02 for the current cert). Let's Encrypt will issue a new cert without downtime.
**Action:** PASS.

## Follow-up Tasks

**T8** (tracked): Replace shared Cloudflare API token with zone-scoped token for `threesix.ai` only (cluster hardening, not a blocker for merge).

## Conclusion

No blocking security issues. The implementation follows cert-manager best practices. The two `ACCEPTED` findings (A2, A4) are inherent to the deployment model and are consistent with the existing cluster security posture. T8 is tracked for future hardening.

**Audit result: PASS**
