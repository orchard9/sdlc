//! Shared utilities for PostgreSQL-backed storage backends.

/// Bridge an async future to a synchronous call.
///
/// When called from a `spawn_blocking` thread (the normal server path), the
/// thread has access to the ambient Tokio runtime handle and `block_on` runs
/// the future on it without spawning a new runtime.
///
/// In CLI contexts where no Tokio runtime is present, a temporary
/// single-threaded runtime is created for the duration of the call.
pub fn block_on_pg<F: std::future::Future>(f: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.block_on(f),
        Err(_) => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create tokio runtime for postgres backend")
            .block_on(f),
    }
}
