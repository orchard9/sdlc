# Audit: ponder-merge-list-ux

## Scope

This feature adds display/filtering logic for merged ponder entries. It reads existing data model fields and adds query parameters, UI filtering, and informational banners. No new write paths, no new authentication surfaces, no new external API calls.

## Findings

### 1. Query parameter injection -- No risk
The `?all=true` query param is deserialized via serde into a `bool`. Non-boolean values default to `false`. No injection surface.

### 2. Client-side filtering -- Defense in depth
The frontend filters merged entries client-side in addition to server-side filtering. This is correct defense-in-depth -- if the server filter is bypassed, the client still hides them by default.

### 3. Slug rendering in banners -- No XSS risk
The `merged_into` slug is rendered in React JSX, which auto-escapes values. The redirect banner in the REST response is a plain string, not HTML. No XSS surface.

### 4. Navigation via merged_into -- Safe
The frontend navigates to `/ponder/${entry.merged_into}` when clicking the redirect banner. Since `merged_into` is a validated slug (alphanumeric + hyphens), there is no path traversal or redirect risk.

### 5. No new data writes
This feature only reads `merged_into` and `merged_from` fields. The actual merge operation (which writes these fields) is in the separate `ponder-merge-cli` feature and is not part of this audit scope.

## Verdict

No security findings. This is a read-only display feature with no new attack surface.
