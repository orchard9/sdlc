# Design: Fix Thread Body Ignored and Submit Button Stays Disabled

## Overview

Two targeted bug fixes with no new abstractions. All changes are surgical — touch only what is broken.

---

## Fix 1 — Persist thread body through the stack

### Data model (`feedback_thread.rs`)

Add `body: Option<String>` to `FeedbackThread`. This field serializes transparently with serde/YAML. Existing manifests without `body` will deserialize with `body: None` (serde default).

```rust
pub struct FeedbackThread {
    pub id: String,
    pub title: String,
    pub context: String,
    pub body: Option<String>,   // ← new
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub post_count: u32,
}
```

Update `create_thread` signature:
```rust
pub fn create_thread(root: &Path, context: &str, title: &str, body: Option<&str>) -> Result<FeedbackThread>
```

Store trimmed, non-empty body on the struct before calling `save_manifest`.

### Route layer (`threads.rs`)

Add `body: Option<String>` to `CreateBody`:
```rust
pub struct CreateBody {
    pub context: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,   // ← new
}
```

Pass it to `create_thread`:
```rust
let body_str = body.body.as_deref().map(str::trim).filter(|s| !s.is_empty());
let thread = sdlc_core::feedback_thread::create_thread(&root, &context, title, body_str)?;
```

Update `thread_to_json` to include `body`:
```rust
"body": t.body.as_deref().map(serde_json::Value::String).unwrap_or(serde_json::Value::Null),
```

Update `get_thread`: remove the hardcoded `value["body"] = serde_json::Value::Null` line. The `thread_to_json` helper now handles it.

### Backward compatibility

- Existing threads on disk have no `body` field → serde deserializes `body: None` → returned as `null`. Safe.
- All existing tests pass unchanged (they pass `body: None`).
- New tests assert body round-trips correctly.

---

## Fix 2 — Re-enable submit button after success

### Modal (`NewThreadModal.tsx`)

**Primary fix** — add `setSubmitting(false)` in the success path:
```ts
try {
  await onSubmit({ title: t, body: body.trim() || undefined })
  setSubmitting(false)   // ← new
} catch (err) {
  setError(...)
  setSubmitting(false)   // existing
}
```

**Safety net** — reset `submitting` when modal reopens:
```ts
useEffect(() => {
  if (open) {
    setTitle('')
    setBody('')
    setError(null)
    setSubmitting(false)   // ← new
    setTimeout(() => titleRef.current?.focus(), 50)
  }
}, [open])
```

Both changes together ensure the button is never stuck regardless of parent close timing.

---

## File Impact

| File | Change |
|---|---|
| `crates/sdlc-core/src/feedback_thread.rs` | Add `body` field to struct; update `create_thread` signature; update tests |
| `crates/sdlc-server/src/routes/threads.rs` | Add `body` to `CreateBody`; pass to core; update `thread_to_json`; remove hardcoded null in `get_thread` |
| `frontend/src/components/threads/NewThreadModal.tsx` | Add `setSubmitting(false)` on success; add to reopen effect |

No schema migration needed. No new endpoints. No new types.
