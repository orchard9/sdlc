# QA Plan: Wildcard TLS cert for *.sdlc.threesix.ai via cert-manager DNS01 in GCP Cloud DNS

## Scope

Verify that the wildcard TLS certificate for `*.sdlc.threesix.ai` is correctly issued, stored, and distributed across the fleet cluster. All test cases are run against the live k3s cluster after applying the `tls/` manifests.

## Test Cases

### TC-1: Namespace Exists

**Type:** Cluster state
**Command:**
```bash
kubectl get namespace sdlc-tls -o jsonpath='{.metadata.labels.app\.kubernetes\.io/managed-by}'
```
**Expected:** Output is `sdlc-cluster`

---

### TC-2: GCP DNS SA ExternalSecret Synced

**Type:** Cluster state
**Command:**
```bash
kubectl get externalsecret gcp-dns-sa-key -n cert-manager -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}'
```
**Expected:** Output is `True`

Verify the resulting Secret has the correct key:
```bash
kubectl get secret gcp-dns-sa-key -n cert-manager -o jsonpath='{.data}' | python3 -c "import sys,json; d=json.load(sys.stdin); print('key.json present' if 'key.json' in d else 'MISSING')"
```
**Expected:** `key.json present`

---

### TC-3: ClusterIssuer Ready

**Type:** Cluster state
**Command:**
```bash
kubectl get clusterissuer letsencrypt-prod -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}'
```
**Expected:** Output is `True`

Verify ACME account registered (no error in message):
```bash
kubectl describe clusterissuer letsencrypt-prod | grep -A5 "Conditions:"
```
**Expected:** `Status: True`, `Type: Ready`, Message does not contain `error` or `failed`

---

### TC-4: Certificate Issued and Ready

**Type:** Cluster state
**Command:**
```bash
kubectl get certificate sdlc-wildcard-tls -n sdlc-tls -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}'
```
**Expected:** Output is `True`

Verify dnsNames on issued cert:
```bash
kubectl get secret sdlc-wildcard-tls -n sdlc-tls -o jsonpath='{.data.tls\.crt}' \
  | base64 -d \
  | openssl x509 -noout -text \
  | grep -A2 "Subject Alternative Name"
```
**Expected:** Output includes `*.sdlc.threesix.ai`

Verify cert not expired and has future expiry:
```bash
kubectl get secret sdlc-wildcard-tls -n sdlc-tls -o jsonpath='{.data.tls\.crt}' \
  | base64 -d \
  | openssl x509 -noout -dates
```
**Expected:** `notAfter` date is in the future (>60 days from test date)

---

### TC-5: Certificate is Let's Encrypt Signed (not self-signed)

**Type:** Certificate validation
**Command:**
```bash
kubectl get secret sdlc-wildcard-tls -n sdlc-tls -o jsonpath='{.data.tls\.crt}' \
  | base64 -d \
  | openssl x509 -noout -issuer
```
**Expected:** Issuer contains `Let's Encrypt` (e.g., `R10`, `E5`, or `R3`)

---

### TC-6: TLS Secret Has Both Required Data Keys

**Type:** Cluster state
**Command:**
```bash
kubectl get secret sdlc-wildcard-tls -n sdlc-tls -o jsonpath='{.data}' \
  | python3 -c "import sys,json; d=json.load(sys.stdin); print(sorted(d.keys()))"
```
**Expected:** `['tls.crt', 'tls.key']`

---

### TC-7: Reflector Auto-Replication to sdlc-* Namespace

**Type:** Cluster state
**Prerequisite:** At least one `sdlc-<slug>` namespace must exist in the cluster.
**Command:**
```bash
# List sdlc-* namespaces (excluding sdlc-tls itself)
kubectl get namespaces -o name | grep "namespace/sdlc-" | grep -v "sdlc-tls"
```

For each discovered `sdlc-<slug>` namespace:
```bash
kubectl get secret sdlc-wildcard-tls -n sdlc-<slug> -o jsonpath='{.type}'
```
**Expected:** `kubernetes.io/tls`

Verify the replicated cert matches the source:
```bash
kubectl get secret sdlc-wildcard-tls -n sdlc-<slug> -o jsonpath='{.data.tls\.crt}' | md5sum
kubectl get secret sdlc-wildcard-tls -n sdlc-tls -o jsonpath='{.data.tls\.crt}' | md5sum
```
**Expected:** Both checksums match.

---

### TC-8: Browser TLS Validation

**Type:** End-to-end
**Prerequisite:** At least one project is deployed with the Helm chart and an Ingress referencing `sdlc-wildcard-tls`. DNS for `<slug>.sdlc.threesix.ai` points to the cluster ingress IP.
**Command:**
```bash
curl -sv https://<slug>.sdlc.threesix.ai 2>&1 | grep -E "SSL connection|certificate|Server certificate|expire date"
```
**Expected:**
- `SSL connection using TLS` present
- `subject: CN=*.sdlc.threesix.ai` present
- No `certificate verify failed` error
- `expire date` is in the future

Alternative using openssl:
```bash
echo | openssl s_client -connect <slug>.sdlc.threesix.ai:443 -servername <slug>.sdlc.threesix.ai 2>/dev/null | openssl x509 -noout -subject -dates
```
**Expected:** `subject=CN=*.sdlc.threesix.ai`, `notAfter` in the future

---

### TC-9: Automatic Renewal Configured

**Type:** Configuration verification
**Command:**
```bash
kubectl get certificate sdlc-wildcard-tls -n sdlc-tls -o jsonpath='{.spec.duration} {.spec.renewBefore}'
```
**Expected:** `2160h 720h` (90-day duration, renew 30 days before expiry)

Verify cert-manager will auto-renew (check for renewal annotation or next renewal time):
```bash
kubectl describe certificate sdlc-wildcard-tls -n sdlc-tls | grep -E "Renewal Time|Not Before|Not After"
```
**Expected:** `Renewal Time` is set to approximately 60 days after issuance.

---

### TC-10: cert-manager Controller No Errors

**Type:** Log inspection
**Command:**
```bash
kubectl logs -n cert-manager -l app=cert-manager --since=5m | grep -i "error\|failed\|sdlc-wildcard-tls" | tail -20
```
**Expected:** No error or failure lines related to `sdlc-wildcard-tls`. Only info/debug lines about successful issuance.

---

## Pass Criteria

All 10 test cases must pass. TC-7 and TC-8 require at least one `sdlc-*` project namespace and a live deployment to be present; if no such namespace exists at QA time, these cases are marked "deferred pending first fleet deployment" and do not block pass/fail.

## Failure Response

| Test | Failure | Recovery |
|---|---|---|
| TC-2 | ESO not syncing | Verify ClusterSecretStore exists; check ESO controller logs |
| TC-3 | ClusterIssuer not ready | Check cert-manager logs; verify SA key JSON is valid |
| TC-4 | Certificate not ready | `kubectl describe certificate` + `kubectl describe challenge -n sdlc-tls` for DNS01 status |
| TC-5 | Self-signed cert | DNS01 challenge failed; check Cloud DNS for TXT record propagation |
| TC-7 | Reflector not replicating | Verify reflector deployment exists; check annotations on the Secret |
| TC-8 | Browser TLS failure | Check if DNS points to correct IP; check Ingress controller logs |
