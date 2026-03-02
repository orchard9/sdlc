## Summary

All 18 QA test cases verified. The `ArtifactViewer.tsx` changes are correct: `extractTeaser` and `formatRelativeTime` behave as specified across all input shapes, the teaser row renders conditionally, and the fullscreen modal is unaffected. The `sdlc-next` and `sdlc-run` command templates contain the `## Summary` convention text in the correct locations.

## Test Results

| Test | Result |
|------|--------|
| TC1: Teaser row — artifact with ## Summary section | PASS |
| TC2: Teaser row — artifact without ## Summary, has body paragraph | PASS |
| TC3: Teaser row — long teaser truncation | PASS |
| TC4: Teaser row — missing artifact | PASS |
| TC5: Timestamp — artifact with approved_at | PASS |
| TC6: Timestamp — artifact with no approved_at | PASS |
| TC7: Teaser row absent when extraction yields empty string | PASS |
| TC8: extractTeaser — H1 skip | PASS |
| TC9: extractTeaser — Summary preference | PASS |
| TC10: formatRelativeTime — seconds | PASS |
| TC11: formatRelativeTime — minutes | PASS |
| TC12: formatRelativeTime — hours | PASS |
| TC13: formatRelativeTime — days | PASS |
| TC14: formatRelativeTime — over a month | PASS |
| TC15: sdlc-next command contains ## Summary instruction | PASS |
| TC16: sdlc-run command contains ## Summary convention callout | PASS |
| TC17: Build and clippy clean | PASS |
| TC18: Fullscreen modal unaffected | PASS |

## Summary Verdict

18/18 test cases PASS. Feature is ready for merge.
