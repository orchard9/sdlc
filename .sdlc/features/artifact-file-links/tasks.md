# Tasks: Auto-detect File Paths as IDE Links

## T1 — Add `Settings` struct and `settings` field to `Config` in `sdlc-core`

In `crates/sdlc-core/src/config.rs`:
- Add `Settings` struct with `ide_uri_scheme: String` field (default `"vscode"`)
- Add `settings: Option<Settings>` field to `Config` with `#[serde(default, skip_serializing_if = "Option::is_none")]`
- Add convenience method `Config::ide_uri_scheme() -> &str`

## T2 — Inject `project_root` into `GET /api/config` response

In `crates/sdlc-server/src/routes/config.rs` `get_config` handler:
- Call `std::env::current_dir()` to get the absolute project root
- Merge it into the JSON response as `"project_root"` key

## T3 — Update frontend `Config` type in `types.ts`

In `frontend/src/lib/types.ts`:
- Add `Settings` interface: `{ ide_uri_scheme: string }`
- Add `settings?: Settings` and `project_root?: string` fields to the `Config` interface

## T4 — Create `ProjectSettingsContext.tsx`

Create `frontend/src/components/shared/ProjectSettingsContext.tsx`:
- Export `ProjectSettingsContext` with `{ projectRoot: string, ideUriScheme: string }` shape
- Default context value: `{ projectRoot: '', ideUriScheme: 'vscode' }`
- Export `useProjectSettings()` hook that calls `useContext(ProjectSettingsContext)`

## T5 — Fetch config and provide `ProjectSettingsContext` in `App.tsx`

In `frontend/src/App.tsx`:
- Fetch `GET /api/config` on mount (once, alongside other startup fetches)
- Extract `project_root` and `settings.ide_uri_scheme` from response
- Wrap app router/children in `<ProjectSettingsContext.Provider value={{ projectRoot, ideUriScheme }}>`

## T6 — Add file-path detection and IDE link rendering to `MarkdownContent.tsx`

In `frontend/src/components/shared/MarkdownContent.tsx`:
- Import `useProjectSettings` from `ProjectSettingsContext`
- Define `FILE_PATH_PATTERN` regex outside the component
- In the `code` component handler, after the existing `isBlock` branch, add file-path detection:
  - Match `FILE_PATH_PATTERN` and require the string contains `/`
  - Require `projectRoot` is non-empty
  - Render `<a href="{ideUriScheme}://file/{projectRoot}/{text}">` with mono styling and underline
  - Fall through to the existing `<code>` span if not a path
