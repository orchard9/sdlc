# Design: orchestrator-action-model

## Module Layout

```
crates/sdlc-core/src/
└── orchestrator/
    ├── mod.rs      — pub use action::*; pub use db::ActionDb;
    ├── action.rs   — Action, ActionTrigger, ActionStatus + serde impls
    └── db.rs       — ActionDb, key encoding, redb table definition, unit tests
```

`lib.rs` gains: `pub mod orchestrator;`

---

## action.rs

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionTrigger {
    Scheduled { next_tick_at: DateTime<Utc> },
    Webhook {
        raw_payload: Vec<u8>,
        received_at: DateTime<Utc>,
    },
}

impl ActionTrigger {
    /// The timestamp used as the redb key prefix.
    pub fn key_ts(&self) -> DateTime<Utc> {
        match self {
            Self::Scheduled { next_tick_at } => *next_tick_at,
            Self::Webhook { received_at, .. } => *received_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionStatus {
    Pending,
    Running,
    Completed { result: serde_json::Value },
    Failed { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: Uuid,
    pub label: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub trigger: ActionTrigger,
    pub status: ActionStatus,
    #[serde(
        serialize_with = "serialize_duration_opt",
        deserialize_with = "deserialize_duration_opt",
        default
    )]
    pub recurrence: Option<Duration>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Action {
    pub fn new_scheduled(
        label: impl Into<String>,
        tool_name: impl Into<String>,
        tool_input: serde_json::Value,
        next_tick_at: DateTime<Utc>,
        recurrence: Option<Duration>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            label: label.into(),
            tool_name: tool_name.into(),
            tool_input,
            trigger: ActionTrigger::Scheduled { next_tick_at },
            status: ActionStatus::Pending,
            recurrence,
            created_at: now,
            updated_at: now,
        }
    }
}

// serde helpers for Duration as seconds (u64)
fn serialize_duration_opt<S>(d: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match d {
        Some(dur) => s.serialize_some(&dur.as_secs()),
        None => s.serialize_none(),
    }
}

fn deserialize_duration_opt<'de, D>(d: D) -> Result<Option<Duration>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<u64> = Option::deserialize(d)?;
    Ok(opt.map(Duration::from_secs))
}
```

---

## db.rs

### Table definition

```rust
use redb::{Database, TableDefinition};
const ACTIONS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("actions");
```

Using `&[u8]` keys (byte slices) gives us full control over the 24-byte composite
key encoding without a custom `redb::Key` implementation.

### Key encoding

```rust
fn action_key(ts: DateTime<Utc>, id: Uuid) -> [u8; 24] {
    let mut key = [0u8; 24];
    let ms = ts.timestamp_millis().max(0) as u64;
    key[..8].copy_from_slice(&ms.to_be_bytes());
    key[8..].copy_from_slice(id.as_bytes());
    key
}

fn due_upper_bound(now: DateTime<Utc>) -> [u8; 24] {
    let mut key = [0u8; 24];
    let ms = now.timestamp_millis().max(0) as u64;
    key[..8].copy_from_slice(&ms.to_be_bytes());
    key[8..].fill(0xff);
    key
}
```

### ActionDb struct

```rust
pub struct ActionDb {
    db: Database,
}

impl ActionDb {
    pub fn open(path: &Path) -> Result<Self> {
        let db = Database::create(path)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        // Ensure table exists
        let wt = db.begin_write()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        wt.open_table(ACTIONS)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        wt.commit()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        Ok(Self { db })
    }

    pub fn insert(&self, action: &Action) -> Result<()> {
        let key = action_key(action.trigger.key_ts(), action.id);
        let value = serde_json::to_vec(action)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let wt = self.db.begin_write()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        {
            let mut table = wt.open_table(ACTIONS)
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            table.insert(key.as_slice(), value.as_slice())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        }
        wt.commit()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        Ok(())
    }

    pub fn set_status(&self, id: Uuid, status: ActionStatus) -> Result<()> {
        // Load all, find by id, delete old key, insert with updated value
        // (status change doesn't change the key; this is safe for current scale)
        let all = self.list_all()?;
        let action = all.into_iter().find(|a| a.id == id)
            .ok_or_else(|| SdlcError::OrchestratorDb(format!("action not found: {id}")))?;
        let old_key = action_key(action.trigger.key_ts(), action.id);
        let mut updated = action;
        updated.status = status;
        updated.updated_at = Utc::now();
        let new_value = serde_json::to_vec(&updated)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let wt = self.db.begin_write()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        {
            let mut table = wt.open_table(ACTIONS)
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            table.remove(old_key.as_slice())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            table.insert(old_key.as_slice(), new_value.as_slice())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        }
        wt.commit()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        Ok(())
    }

    pub fn range_due(&self, now: DateTime<Utc>) -> Result<Vec<Action>> {
        let upper = due_upper_bound(now);
        let rt = self.db.begin_read()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let table = rt.open_table(ACTIONS)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let mut result = Vec::new();
        for entry in table.range(..=upper.as_slice())
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?
        {
            let (_, v) = entry.map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let action: Action = serde_json::from_slice(v.value())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            if matches!(action.status, ActionStatus::Pending) {
                result.push(action);
            }
        }
        Ok(result)
    }

    pub fn startup_recovery(&self, max_age: Duration) -> Result<u32> {
        let cutoff = Utc::now() - chrono::Duration::from_std(max_age)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let all = self.list_all()?;
        let mut count = 0u32;
        for action in all {
            if matches!(action.status, ActionStatus::Running)
                && action.updated_at < cutoff
            {
                self.set_status(
                    action.id,
                    ActionStatus::Failed {
                        reason: "recovered from restart".into(),
                    },
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn list_all(&self) -> Result<Vec<Action>> {
        let rt = self.db.begin_read()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let table = rt.open_table(ACTIONS)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let mut result = Vec::new();
        for entry in table.iter()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?
        {
            let (_, v) = entry.map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let action: Action = serde_json::from_slice(v.value())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            result.push(action);
        }
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }
}
```

---

## Error variant

Add to `crates/sdlc-core/src/error.rs`:

```rust
#[error("orchestrator DB error: {0}")]
OrchestratorDb(String),
```

---

## Cargo changes

`Cargo.toml` (workspace):
```toml
redb = "2"
```

`crates/sdlc-core/Cargo.toml`:
```toml
redb = { workspace = true }
uuid = { workspace = true }
```

---

## Key design trade-offs

| Decision | Rationale |
|---|---|
| `&[u8]` redb key type | Avoids custom `redb::Key` impl; 24-byte array gives full control |
| `set_status` does delete+insert | Key doesn't change on status update; simpler than a secondary index |
| Filter Pending in `range_due` | Keeps Running/Completed out of the tick loop without a second table |
| `list_all` full scan | Acceptable for current scale; add indexing when actions exceed 10k |
