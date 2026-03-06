# QA Plan: hub-create-repo-ui

## Build Check

`cd frontend && npm run build` — must complete without TypeScript errors.

## Manual UI Scenarios

| Scenario | Steps | Expected |
|---|---|---|
| Happy path | Type `my-project`, click Create | Step 2 shown with two copy commands |
| Copy remote | Click Copy on Add remote row | Clipboard has full push URL |
| Copy push | Click Copy on Push row | Clipboard has `git push gitea main` |
| Copy feedback | Click Copy | Button shows "Copied!" for ~1.5s then resets |
| Add another | Click Add another project | Returns to Step 1, input cleared |
| Invalid name — empty | Click Create with empty input | Button stays disabled |
| Invalid name — uppercase | Type `MyProject` | Inline error: "lowercase letters, numbers, and hyphens" |
| Invalid name — spaces | Type `my project` | Same inline error |
| API conflict error | Server returns 409 | Shows "A repo named X already exists" error |
| Generic API error | Server returns 500 | Shows error message, stays on Step 1 |
| Creating state | After click, before response | Input disabled, spinner shown |

## TypeScript

No `any` casts. `CreateRepoResponse` fields fully typed. `api.createRepo` return type inferred.
