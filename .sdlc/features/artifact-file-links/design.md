# Design: Auto-detect File Paths as IDE Links

## Overview

This feature adds file-path detection to `MarkdownContent.tsx` so that inline code spans containing file paths (e.g., `` `crates/sdlc-core/src/feature.rs` ``) become clickable IDE links. The link URI is constructed from a configurable `ide_uri_scheme` and the server's `project_root`.

The change spans three layers:
1. **Config** (`sdlc-core`) — add `settings.ide_uri_scheme` field
2. **Server** (`sdlc-server`) — expose `project_root` + `ide_uri_scheme` via API
3. **Frontend** (`frontend`) — detect file paths in inline code and render as links

---

## Component Architecture

```
Config.yaml
  └── settings.ide_uri_scheme: "vscode"  (default)

sdlc-server GET /api/config
  └── returns: { ..., settings: { ide_uri_scheme: "vscode" }, project_root: "/abs/path" }

React App (App.tsx or useProjectInfo hook)
  └── fetches /api/config once at startup
  └── provides { projectRoot, ideUriScheme } via context or prop drilling

MarkdownContent.tsx
  └── props: { content, className, projectRoot?, ideUriScheme? }
  └── inline code handler detects FILE_PATH_PATTERN
  └── renders <a href="{scheme}://file/{root}/{path}"> when matched
```

---

## Data Flow

### 1. Config Layer (`sdlc-core`)

Add a `Settings` struct and `settings` field to `Config`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    /// URI scheme for IDE deep links. Supported: vscode, cursor, zed, idea.
    /// Default: "vscode"
    #[serde(default = "default_ide_scheme")]
    pub ide_uri_scheme: String,
}

fn default_ide_scheme() -> String {
    "vscode".to_string()
}

// In Config struct:
#[serde(default, skip_serializing_if = "Option::is_none")]
pub settings: Option<Settings>,
```

`Config::ide_uri_scheme()` convenience method returns `settings.as_ref().map(|s| s.ide_uri_scheme.as_str()).unwrap_or("vscode")`.

### 2. Server Layer (`sdlc-server`)

In `routes/config.rs` `get_config` handler, augment the response with `project_root`:

```rust
let project_root = std::env::current_dir()
    .unwrap_or_default()
    .to_string_lossy()
    .to_string();

Ok(Json(serde_json::json!({
    ...config_json,
    "project_root": project_root,
})))
```

`project_root` is the working directory at server startup — this is always the project root because `sdlc ui` is run from the project directory.

### 3. Frontend: Settings Consumption

**`frontend/src/lib/types.ts`** — add to `Config` type:

```ts
export interface Settings {
  ide_uri_scheme: string;
}

export interface Config {
  // ... existing fields ...
  settings?: Settings;
  project_root?: string;  // injected by server
}
```

**`frontend/src/App.tsx` or a `useProjectSettings` hook** — fetch config once and store in context or app-level state. Pass `projectRoot` and `ideUriScheme` down to all `<MarkdownContent>` usages.

A lightweight approach: expose via React context `ProjectSettingsContext` so `MarkdownContent` can consume it without prop threading through every call site.

```ts
// ProjectSettingsContext.tsx
export const ProjectSettingsContext = React.createContext({
  projectRoot: '',
  ideUriScheme: 'vscode',
});
```

Provider wraps the app root; `MarkdownContent` calls `useContext(ProjectSettingsContext)` internally — no prop change needed at call sites.

### 4. Frontend: `MarkdownContent.tsx`

Add file-path detection in the `code` component handler. The pattern must be conservative to avoid false positives on CSS classes, numbers, etc.:

```tsx
// Matches: crates/foo/bar.rs, src/lib/types.ts, Cargo.toml, README.md
// Does NOT match: max-h-96, Some(20), #selector, --flag
const FILE_PATH_PATTERN = /^[a-zA-Z_.][a-zA-Z0-9_\-./]*\.[a-zA-Z]{2,5}$/;

// Additional guard: must contain at least one path separator OR be a known filename
const hasPathSeparator = (s: string) => s.includes('/');
```

Only strings that match the pattern AND contain a `/` are treated as paths. This prevents false positives on simple filenames that might be code tokens.

In the `code` component:

```tsx
code: ({ children, className: codeClass, node }) => {
  const lang = codeClass?.replace('language-', '') ?? '';
  const isBlock = node?.position?.start.line !== node?.position?.end.line;
  const { projectRoot, ideUriScheme } = useContext(ProjectSettingsContext);

  if (lang === 'mermaid') return <MermaidBlock chart={String(children).trim()} />;
  if (lang) return <SyntaxHighlighter ...>{String(children).trim()}</SyntaxHighlighter>;
  if (isBlock) return <pre ...><code>{children}</code></pre>;

  // File path detection (inline code only)
  const text = String(children);
  if (FILE_PATH_PATTERN.test(text) && hasPathSeparator(text) && projectRoot) {
    const href = `${ideUriScheme}://file/${projectRoot}/${text}`;
    return (
      <a
        href={href}
        className="font-mono text-xs text-primary underline underline-offset-2 hover:opacity-80 transition-opacity"
        title={`Open in IDE: ${text}`}
      >
        {text}
      </a>
    );
  }

  return <code className="text-xs font-mono bg-muted/60 border border-border/50 px-1 py-0.5 rounded text-muted-foreground">{children}</code>;
},
```

Note: `useContext` inside component handlers defined inline in `components` prop is valid because the outer `MarkdownContent` component is a React function component — the handlers close over the context value from the outer render.

---

## File Path Pattern Analysis

| Input | Matches? | Expected |
|---|---|---|
| `crates/sdlc-core/src/feature.rs` | yes | link |
| `frontend/src/lib/types.ts` | yes | link |
| `Cargo.toml` | no (no `/`) | code span |
| `max-h-96` | no (no `.ext`) | code span |
| `Some(20)` | no (`(` not in pattern) | code span |
| `#selector` | no (`#` not in pattern) | code span |
| `--flag` | no (no `.ext`) | code span |
| `sdlc_core::feature` | no (`::` and no `.ext`) | code span |
| `.sdlc/config.yaml` | yes (starts with `.`) | link |

The `/` requirement is the primary guard. Virtually no code token has a forward slash; file paths almost always do.

---

## ASCII Wireframe

```
Before:
┌─────────────────────────────────────────────────────┐
│  The main logic lives in `crates/sdlc-core/src/     │
│  feature.rs` and is called by `sdlc next`.          │
│                                                     │
│  ┌──────────────────────────────────┐               │
│  │ crates/sdlc-core/src/feature.rs  │  ← inert code │
│  └──────────────────────────────────┘               │
└─────────────────────────────────────────────────────┘

After:
┌─────────────────────────────────────────────────────┐
│  The main logic lives in `crates/sdlc-core/src/     │
│  feature.rs` and is called by `sdlc next`.          │
│                                                     │
│  ┌──────────────────────────────────┐               │
│  │ crates/sdlc-core/src/feature.rs  │  ← clickable  │
│  └──────────────────────────────────┘    IDE link   │
└─────────────────────────────────────────────────────┘
```

---

## API Response Shape

`GET /api/config` response (augmented):

```json
{
  "version": 1,
  "project": { "name": "sdlc" },
  "settings": {
    "ide_uri_scheme": "vscode"
  },
  "project_root": "/Users/jordan/Workspace/orchard9/sdlc",
  "phases": { ... },
  "platform": null,
  "quality": null
}
```

---

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-core/src/config.rs` | Add `Settings` struct + `settings` field to `Config` |
| `crates/sdlc-server/src/routes/config.rs` | Inject `project_root` into `get_config` response |
| `frontend/src/lib/types.ts` | Add `Settings` interface + `settings`/`project_root` to `Config` |
| `frontend/src/components/shared/MarkdownContent.tsx` | File path detection + IDE link rendering |
| `frontend/src/` (new file) | `ProjectSettingsContext.tsx` — React context for projectRoot + ideUriScheme |
| `frontend/src/App.tsx` | Fetch config, provide `ProjectSettingsContext` |

---

## Edge Cases & Decisions

**What if `project_root` is empty?** The file path detection guard `&& projectRoot` short-circuits — paths render as normal code spans. No broken links.

**What if `ide_uri_scheme` is not in config?** Default is `"vscode"`. The `Settings` struct uses `#[serde(default)]` and `default_ide_scheme()`.

**Does this affect fenced code blocks?** No. The `lang` check fires first; blocks with a language are handled by `SyntaxHighlighter`. Blocks without a language use the `isBlock` check. Only inline spans reach the file-path branch.

**Performance?** The regex is compiled once as a `const`/`static` using `once_cell` or defined outside the component. No per-render allocation.

**Accessibility?** The link has a `title` attribute describing the action. The `font-mono` class preserves code appearance while the underline signals interactivity.
