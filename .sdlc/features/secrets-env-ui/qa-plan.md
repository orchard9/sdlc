# QA Plan: Secrets — Add Environment Modal and CLI Hint Affordances

## Scope

Verify that the Add Environment modal and CLI hint affordances work correctly on the SecretsPage without regressions.

## Test Scenarios

### QA-1: Add Environment button visibility
- Navigate to the Secrets page.
- Verify the Environments section header contains an "Add Environment" button (Plus icon + text).
- Verify the button is visible both when no envs exist (empty state) and when envs are present.

### QA-2: Modal opens and closes
- Click "Add Environment".
- Verify the modal appears with: env name input, one empty key-value row, "Add row" button, "Cancel" and "Create Environment" buttons.
- Click X or Cancel — verify modal closes without side effects.

### QA-3: Client-side validation
- Submit with empty env name — verify inline error "Name and secret are required" (or similar) and no API call made.
- Submit with env name but empty key — verify validation error and no API call.
- Submit with env name and a key but empty value — verify it proceeds (empty values are allowed).

### QA-4: Add row and remove row
- Click "Add row" — verify a second key-value row appears.
- Click the trash icon on the second row — verify it is removed.
- Verify the trash icon on the only remaining row is disabled.

### QA-5: Successful environment creation
- Ensure at least one authorized key is configured (prerequisite).
- Open the modal, enter env name "qa-test", add a pair KEY=`TEST_VAR`, VALUE=`hello`.
- Submit.
- Verify modal closes.
- Verify the env list updates to include "qa-test".
- Verify the new card shows the correct env name and key count.

### QA-6: Duplicate environment name (409 error)
- Create an env named "qa-test" (or use existing one).
- Open the modal, enter the same env name, add a pair.
- Submit.
- Verify inline error message containing "already exists".
- Verify modal stays open.

### QA-7: No keys configured (422 error)
- If possible (test environment with no keys), attempt to create an env.
- Verify appropriate error is shown inline.

### QA-8: CLI export hint (existing) — no regression
- Verify each existing env card still shows the `eval $(sdlc secrets env export <env>)` hint with copy button.

### QA-9: CLI set-secret hint (new)
- Verify each existing env card shows a `sdlc secrets env set <env> KEY=value` hint with copy button.
- Verify the copy button copies the correct text (with the specific env name, not a placeholder).

### QA-10: Delete env — no regression
- Verify the trash/delete button on env cards still functions correctly.
- Verify deleted envs disappear from the list.

### QA-11: Key management — no regression
- Verify the Authorized Keys section is unaffected: Add Key, Remove Key, rekey hint all function normally.

## Automated Tests

### Unit tests (if applicable)
- `AddEnvModal` validation logic can be tested in isolation with vitest/react-testing-library.

### Build verification
- `SDLC_NO_NPM=1 cargo test --all` — no Rust regressions.
- `cd frontend && npm run build` — TypeScript compiles without errors.

## Pass Criteria

All scenarios QA-1 through QA-11 pass. Build is clean. No TypeScript errors.
