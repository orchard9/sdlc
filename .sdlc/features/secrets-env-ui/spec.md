# Spec: Secrets — Add Environment Modal and CLI Hint Affordances

## Problem

The SecretsPage Environments section is read-only from the dashboard: users can list, delete, and export envs, but cannot create a new encrypted env file without using the CLI. The current empty state shows only a static code snippet — no interactive affordance. Experienced users know to use the CLI, but the page gives no clear guidance for adding secrets to existing envs either.

## Goal

1. Allow users to create a new encrypted environment directly from the SecretsPage via an "Add Environment" modal.
2. Add a CLI hint affordance on each existing environment card so users can easily copy the command for setting additional secrets.

## Scope

This feature covers frontend-only UI changes on `SecretsPage.tsx`. It depends on the `POST /api/secrets/envs` endpoint delivered by `secrets-create-env-endpoint`.

## User Stories

### US-1: Create a new environment from the dashboard
As a user, I can click "Add Environment" on the Environments section header, fill in an environment name and one or more KEY=VALUE pairs, and submit — creating an encrypted env file visible in the list without leaving the browser.

### US-2: CLI hint for adding secrets to an existing env
As a user, each existing environment card shows a copy button for the `sdlc secrets env set <env> KEY=value` command so I can add secrets to that env from my terminal without having to remember the syntax.

## Functional Requirements

### FR-1: Add Environment button
- The Environments section header gains a "Add Environment" button (same style as the existing "Add Key" button in the Keys section).
- The button is present regardless of whether any envs exist.

### FR-2: Add Environment modal
- The modal contains:
  - An **env name** text input (required, validated as non-empty, alphanumeric/hyphen/underscore/dot).
  - A **key-value list**: one or more rows of `KEY` + `VALUE` text inputs with an "Add Row" button.
  - At least one key-value pair is required on submission.
  - Submit calls `POST /api/secrets/envs` with `{ env, pairs: [{ key, value }] }`.
  - On success: modal closes, env list refreshes via SSE (no manual `refresh()` call needed — SSE triggers it).
  - On 409 Conflict: shows inline error "An environment named '<name>' already exists".
  - On other errors: shows inline error with the server message.
  - Loading state disables the submit button with a spinner.
- The modal follows the existing `AddKeyModal` pattern (fixed overlay, card container, X close button).

### FR-3: CLI hint on environment cards
- Each existing environment card gets a "Set a secret via CLI" hint row below the existing `eval $(sdlc secrets env export …)` copy row.
- Format: `sdlc secrets env set <env> KEY=value` with a copy button.
- This hint is always visible (not collapsed).

### FR-4: API client method
- A new `createSecretsEnv(body: { env: string; pairs: { key: string; value: string }[] })` method added to `api/client.ts` calling `POST /api/secrets/envs`.

## Non-Goals

- No server-side decryption. The `POST /api/secrets/envs` endpoint encrypts using configured public keys only — no identity/private key required.
- No UI for setting secrets in an existing env (update flow) — that requires a private key for re-encryption and belongs to a future feature.
- No env rename or key-level edit from the UI.

## Design Notes

- The Add Environment button sits in the section header alongside the count badge, mirroring the Authorized Keys section layout.
- Modal field order: env name first, then key-value pairs (minimum one row pre-populated).
- The "Add Row" button is a small muted text button at the bottom of the key-value list.
- Rows can be removed (trash icon) except when there is only one row left.
- Key inputs should be uppercase-hinted (placeholder `MY_API_KEY`).

## Acceptance Criteria

1. An "Add Environment" button appears in the Environments section header on SecretsPage.
2. Clicking it opens a modal with env name input and at least one KEY/VALUE row.
3. Submitting with a valid name and at least one key-value pair calls the API and closes the modal on success.
4. The environment list updates after creation (via SSE).
5. Submitting an env name that already exists shows a 409 conflict error inline.
6. Each existing environment card shows a copy-button CLI hint for `sdlc secrets env set <env> KEY=value`.
7. No regressions to existing key management, env list, or delete functionality.
