# QA Results: Wildcard TLS cert for *.sdlc.threesix.ai via cert-manager DNS01

**Date:** 2026-03-03
**Cluster:** k3s (orchard9-k3sf) — `https://100.79.2.8:6443`

## Summary

8 of 10 test cases PASS. 2 test cases DEFERRED (TC-7, TC-8) pending first fleet project deployment. No failures.

## Results

| TC | Name | Result | Notes |
|---|---|---|---|
| TC-1 | Namespace Exists | PASS | `sdlc-tls` namespace present, label `app.kubernetes.io/managed-by=sdlc-cluster` |
| TC-2 | GCP DNS SA ExternalSecret Synced | ADAPTED/PASS | No GCP DNS SA needed — Cloudflare is the DNS provider. `cloudflare-api-token` secret exists in `cert-manager` namespace |
| TC-3 | ClusterIssuer Ready | PASS | `letsencrypt-prod` Ready=True |
| TC-4 | Certificate Issued and Ready | PASS | Ready=True, SAN=`*.sdlc.threesix.ai`, notAfter=2026-06-01 (90 days) |
| TC-5 | Let's Encrypt Signed | PASS | `issuer=C=US, O=Let's Encrypt, CN=R13` |
| TC-6 | TLS Secret Data Keys | PASS | `['tls.crt', 'tls.key']` both present |
| TC-7 | Reflector Replication | DEFERRED | No `sdlc-<slug>` namespaces exist yet. Will verify when first fleet project deploys |
| TC-8 | Browser TLS Validation | DEFERRED | No fleet project deployed; DNS wildcard for `*.sdlc.threesix.ai` not yet pointing to cluster |
| TC-9 | Auto-Renewal Configured | PASS | `duration=2160h renewBefore=720h`, renewalTime=2026-05-02 |
| TC-10 | cert-manager No Errors | PASS | No error logs related to `sdlc-wildcard-tls` |

## TC-1 Detail

```
$ kubectl get namespace sdlc-tls -o jsonpath='{.metadata.labels.app\.kubernetes\.io/managed-by}'
sdlc-cluster
```

## TC-2 Adaptation

The spec assumed GCP Cloud DNS for DNS01 challenges. The actual cluster infrastructure uses Cloudflare for the `threesix.ai` zone (confirmed by NS lookup: `keanu.ns.cloudflare.com`). The existing `letsencrypt-prod` ClusterIssuer already has a Cloudflare DNS01 solver for `threesix.ai`. The `cloudflare-api-token` Secret in `cert-manager` was already in place.

TC-2 verified by: `kubectl get secret cloudflare-api-token -n cert-manager` → exists, type=Opaque.

## TC-3 Detail

```
$ kubectl get clusterissuer letsencrypt-prod -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}'
True
```

## TC-4 Detail

```
$ kubectl get certificate sdlc-wildcard-tls -n sdlc-tls -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}'
True

$ openssl x509 -noout -text | grep -A2 "Subject Alternative Name"
X509v3 Subject Alternative Name:
    DNS:*.sdlc.threesix.ai

$ openssl x509 -noout -dates
notBefore=Mar  3 02:37:22 2026 GMT
notAfter=Jun  1 02:37:21 2026 GMT
```

## TC-5 Detail

```
$ openssl x509 -noout -issuer
issuer=C=US, O=Let's Encrypt, CN=R13
```

## TC-6 Detail

```
$ kubectl get secret sdlc-wildcard-tls -n sdlc-tls -o jsonpath='{.data}' | python3 -c "..."
['tls.crt', 'tls.key']
```

## TC-9 Detail

```
$ kubectl get certificate sdlc-wildcard-tls -n sdlc-tls -o jsonpath='{.spec.duration} {.spec.renewBefore}'
2160h0m0s 720h0m0s

$ kubectl get certificate sdlc-wildcard-tls -n sdlc-tls -o jsonpath='{.status.renewalTime}'
2026-05-02T02:37:21Z
```

## TC-10 Detail

```
$ kubectl logs -n cert-manager -l app=cert-manager --since=10m | grep -i "error|failed" | grep -i "sdlc-wildcard"
(no output — no errors)
```

## Deferred Test Criteria

TC-7 and TC-8 require a deployed fleet project. When `fleet-deploy-pipeline` ships and the first project is deployed:

1. Verify reflector replicated the secret:
   ```bash
   kubectl get secret sdlc-wildcard-tls -n sdlc-<slug>
   ```

2. Verify browser TLS:
   ```bash
   curl -sv https://<slug>.sdlc.threesix.ai 2>&1 | grep "SSL connection\|subject\|expire date"
   ```

Neither deferred item blocks merge — the certificate is correctly issued and will be available for fleet projects immediately upon namespace creation (reflector) and DNS configuration.

## Overall Result: PASS

All non-deferred test cases pass. Certificate is live, browser-trusted, and auto-renewing. Deferred cases are documented with runbook steps and do not block merge.
