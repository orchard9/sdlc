use crate::config::Config;
use crate::error::Result;

/// Run any pending schema migrations on a loaded [`Config`].
///
/// Currently a no-op — schema v1 has no migrations. When the config schema changes in ways
/// that require data transforms, add a match arm here:
///
/// ```rust,ignore
/// match cfg.version {
///     1 => migrate_v1_to_v2(cfg),
///     _ => Ok(cfg),
/// }
/// ```
pub fn migrate_config(cfg: Config) -> Result<Config> {
    Ok(cfg)
}
