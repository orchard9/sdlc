# Security Audit: Park/Unpark Core Data Model

## Scope

Pure data model change in `sdlc-core`: new enum variant, new `Option<DateTime>` field, two setter methods, updated status derivation logic.

## Findings

### A1: No new attack surface
The changes are entirely within the core library data layer. No new CLI commands, REST endpoints, or user-facing inputs are introduced in this feature. Those are scoped to `park-milestone-cli-api`. **No action needed.**

### A2: No authorization changes
The `park()` and `unpark()` methods are simple field setters with no access control. Authorization is enforced at the CLI/REST layer (separate feature). **No action needed.**

### A3: No file system access changes
The `parked_at` field is serialized/deserialized through the existing `Milestone::save`/`Milestone::load` path using `atomic_write`. No new file paths or I/O operations. **No action needed.**

### A4: No deserialization risk
The field uses `serde(default)` which safely handles missing values. No custom deserializer. No untrusted input parsing. **No action needed.**

### A5: Backward compatibility — no data corruption risk
Existing milestone YAML files without `parked_at` load correctly (defaults to `None`). Files with `parked_at` set will have it ignored by older versions (serde ignores unknown fields by default in YAML). **No action needed.**

## Verdict

No security findings. This is a minimal, additive data model change with no security-relevant surface area.
