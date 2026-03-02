## Summary

Extend `MarkdownContent.tsx`'s inline `code` component handler to detect file path patterns and render them as `{ide}://file/{project_root}/{path}` clickable links. Requires adding `project_root` to the server API response and an `ide_uri_scheme` config field. This eliminates the second half of Xist's Agy workaround — file references in plans that open in the IDE.

## Problem

Xist: "I always make Agy include links in my plans, so all source files/classes named in the plans are linked to their actual source location and I can open them up in Agy and look at them." SDLC agents write file references as inline code (e.g., `` `crates/sdlc-core/src/feature.rs` ``) that render as inert code spans with no click behavior.

## Solution

### Frontend: `MarkdownContent.tsx`

In the inline `code` component handler (which already dispatches on inline vs. block code spans), add a file-path detection branch:

```tsx
const FILE_PATH_PATTERN = /^[a-z_.][a-z0-9_\-./]*\.[a-z]{2,5}$/i;

// In the `code` component:
const isInline = !className; // react-markdown sets className for fenced code blocks
if (isInline && FILE_PATH_PATTERN.test(children as string)) {
  const filePath = children as string;
  const href = `${ideUriScheme}://file/${projectRoot}/${filePath}`;
  return (
    <a href={href} className="font-mono text-sm text-primary underline hover:opacity-80" title={`Open in IDE: ${filePath}`}>
      {filePath}
    </a>
  );
}
```

`projectRoot` and `ideUriScheme` are passed as props to `MarkdownContent`.

### Backend: Add `project_root` to API

In `crates/sdlc-core/src/` (project state struct or new `ProjectInfo` struct), add:
```rust
pub project_root: String, // std::env::current_dir() at server startup
```

Expose via the existing project state API endpoint (check `crates/sdlc-server/src/routes/` for the right endpoint — likely `/api/state` or `/api/project`).

### Config: `ide_uri_scheme`

Add to `.sdlc/config.yaml` under a `settings` key:
```yaml
settings:
  ide_uri_scheme: vscode  # Options: vscode, cursor, zed, idea
```

Server reads this field and includes it in the API response alongside `project_root`. Default is `vscode` if field is absent.

### Frontend: Settings consumption

In the React app, read `project_root` and `ide_uri_scheme` from the project state API response and pass them into every `<MarkdownContent>` component. The values are available at app initialization and don't change at runtime.

## Acceptance Criteria

- Inline code span `` `crates/sdlc-core/src/feature.rs` `` in a plan artifact renders as a clickable link
- Clicking the link triggers a `vscode://file/{cwd}/crates/sdlc-core/src/feature.rs` URI (or equivalent for configured IDE)
- Non-path inline code (e.g., `` `max-h-96` ``, `` `Some(20)` ``) renders as normal code span (not a link)
- Code blocks (fenced with ```) are unaffected — they do not get link treatment
- Setting `ide_uri_scheme: cursor` in config causes links to use `cursor://file/...` scheme
- `project_root` is present in the server API response

## Files Changed

- `frontend/src/components/shared/MarkdownContent.tsx` — file path detection and link rendering
- `frontend/src/` (app root or settings hook) — read and propagate `project_root` + `ide_uri_scheme`
- `crates/sdlc-core/src/` — add `project_root` to project state struct
- `crates/sdlc-server/src/routes/` — expose `project_root` and `ide_uri_scheme` in API response

## What is NOT in scope

- Visual differentiation of IDE links vs. web links (V1.1 — no file icon overlay)
- Verifying whether a detected path actually exists on disk (broken links fail silently in VS Code — acceptable for V1)
- Detecting file paths in formats other than inline code (e.g., `(parenthesized/path.rs)` in link syntax)
