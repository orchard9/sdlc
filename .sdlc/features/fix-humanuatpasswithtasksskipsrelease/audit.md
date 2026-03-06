# Audit: Fix Human UAT PassWithTasks Skips Release

## Security

- **No new attack surface.** The change only broadens an existing passing-verdict check — no new inputs, no new endpoints.
- **No injection risk.** The verdict value comes from a typed enum (`UatVerdict`), deserialized by serde.

## Correctness

- **Fix matches the root cause exactly.** The conditional now includes `PassWithTasks` alongside `Pass`, which is the only missing case.
- **No behavioral change for other verdicts.** `Failed` still does not trigger release. `Pass` still works as before.
- **Notes validation preserved.** `PassWithTasks` still requires non-empty notes (line 1165 check is unaffected).

## Code Quality

- Uses idiomatic `matches!` macro — extensible if more passing variants are added later.
- Test follows existing patterns in the integration test suite.
- Comment updated to reflect the broadened condition.

## Findings

None. The change is minimal and correct.
