use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;
use std::path::Path;

use crate::state::AppState;

#[derive(Embed)]
#[folder = "$SDLC_FRONTEND_DIST"]
struct FrontendAssets;

/// Compute the page title from the project's .sdlc/state.yaml.
/// Returns "sdlc — {project}" when a project name is present,
/// or "sdlc" as a safe fallback when the name is absent or unreadable.
fn compute_title(root: &Path) -> String {
    let project = sdlc_core::state::State::load(root)
        .ok()
        .map(|s| s.project)
        .unwrap_or_default();

    if project.is_empty() {
        "sdlc".to_string()
    } else {
        format!("sdlc \u{2014} {project}")
    }
}

/// Replace the contents of the `<title>` tag in `html` with `title`.
/// Returns the original string unchanged if no `<title>` tag is found.
fn inject_title(html: &str, title: &str) -> String {
    if let Some(start) = html.find("<title>") {
        if let Some(end_offset) = html[start..].find("</title>") {
            let tag_end = start + end_offset + "</title>".len();
            return format!(
                "{}<title>{}</title>{}",
                &html[..start],
                title,
                &html[tag_end..]
            );
        }
    }
    // No <title> tag found — return unchanged
    html.to_string()
}

/// Serve embedded frontend assets. Falls back to index.html for SPA routing.
/// Injects a dynamic page title ("sdlc — {project-name}") into index.html at
/// serve time so browser tabs reflect the current project.
pub async fn static_handler(State(app): State<AppState>, uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    // Try the exact path first (static assets: JS, CSS, images, etc.)
    if let Some(content) = <FrontendAssets as Embed>::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime.as_ref())],
            content.data.to_vec(),
        )
            .into_response();
    }

    // SPA fallback: serve index.html with injected project title
    match <FrontendAssets as Embed>::get("index.html") {
        Some(content) => {
            let html = String::from_utf8_lossy(&content.data).into_owned();
            let title = compute_title(&app.root);
            let html = inject_title(&html, &title);
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/html")],
                html.into_bytes(),
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "frontend not built").into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_title_replaces_existing_title() {
        let html = "<html><head><title>Ponder</title></head><body></body></html>";
        let result = inject_title(html, "sdlc \u{2014} myapp");
        assert_eq!(
            result,
            "<html><head><title>sdlc \u{2014} myapp</title></head><body></body></html>"
        );
    }

    #[test]
    fn inject_title_no_title_tag_returns_unchanged() {
        let html = "<html><head></head><body></body></html>";
        let result = inject_title(html, "sdlc \u{2014} myapp");
        assert_eq!(result, html);
    }

    #[test]
    fn inject_title_replaces_sdlc_base_title() {
        let html = "<html><head><title>sdlc</title></head></html>";
        let result = inject_title(html, "sdlc \u{2014} testproject");
        assert_eq!(
            result,
            "<html><head><title>sdlc \u{2014} testproject</title></head></html>"
        );
    }

    #[test]
    fn compute_title_falls_back_to_sdlc_for_missing_state() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let title = compute_title(tmp.path());
        assert_eq!(title, "sdlc");
    }

    #[test]
    fn compute_title_with_project_name() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".sdlc")).expect("create .sdlc");
        // Write a minimal state.yaml with a project name
        std::fs::write(
            tmp.path().join(".sdlc").join("state.yaml"),
            "version: 1\nproject: myapp\nactive_features: []\nactive_directives: []\nhistory: []\nblocked: []\nmilestones: []\nactive_ponders: []\nlast_updated: \"2026-01-01T00:00:00Z\"\n",
        ).expect("write state.yaml");
        let title = compute_title(tmp.path());
        assert_eq!(title, "sdlc \u{2014} myapp");
    }

    #[test]
    fn compute_title_with_empty_project_name() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".sdlc")).expect("create .sdlc");
        std::fs::write(
            tmp.path().join(".sdlc").join("state.yaml"),
            "version: 1\nproject: \"\"\nactive_features: []\nactive_directives: []\nhistory: []\nblocked: []\nmilestones: []\nactive_ponders: []\nlast_updated: \"2026-01-01T00:00:00Z\"\n",
        ).expect("write state.yaml");
        let title = compute_title(tmp.path());
        assert_eq!(title, "sdlc");
    }
}
