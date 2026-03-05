fn main() {
    // Append the git tag (from GitHub Actions) to the Cargo version string.
    // Output: "0.1.0 (tag: v0.1.0-2)" or "0.1.0 (untagged)".
    let cargo_version = env!("CARGO_PKG_VERSION");
    let tag_suffix = std::env::var("GITHUB_REF_NAME")
        .ok()
        .filter(|r| r.starts_with('v'))
        .map(|r| format!("tag: {r}"))
        .unwrap_or_else(|| "untagged".to_string());

    println!("cargo:rustc-env=SDLC_GIT_VERSION={cargo_version} ({tag_suffix})");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");
}
