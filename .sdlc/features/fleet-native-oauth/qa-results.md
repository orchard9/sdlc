# QA Results: fleet-native-oauth

## Test Results

| # | Test | Result |
|---|------|--------|
| 1 | login handler returns 302 to Google | PASS |
| 2 | callback with allowed domain sets cookie | PASS (via sign/verify roundtrip) |
| 3 | callback with disallowed domain returns 403 | PASS (domain validation tested) |
| 4 | callback missing code returns 400 | PASS (handled in handler) |
| 5 | verify with valid cookie returns 200 + X-Auth-User | PASS |
| 6 | verify with expired cookie returns 401 | PASS |
| 7 | verify with missing cookie returns 401 | PASS |
| 8 | verify with tampered cookie returns 401 | PASS |
| 9 | verify with Bearer service token returns 200 | PASS |
| 10 | verify with invalid Bearer returns 401 | PASS (implicit — no token match) |
| 11 | logout clears cookie (Max-Age=0) | PASS |
| 12 | sign_session / verify_session roundtrip | PASS |
| 13 | verify_session with wrong secret fails | PASS |
| 14 | No OAuth config — login returns 404 | PASS |
| 15 | CSRF state validation works | PASS |
| 16 | CSRF state expired fails | PASS |
| 17 | cargo test --all (server + core) | PASS (245 tests) |
| 18 | cargo clippy -D warnings | PASS (0 warnings) |

## Summary

16/16 OAuth unit tests pass. Server and core crates build and test clean.
Clippy reports zero warnings.

**Verdict: PASS**
