use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let frontend_dir = manifest_dir.join("../../frontend");
    let real_dist = frontend_dir.join("dist");

    // Rerun when frontend sources change, or when dist/index.html appears or
    // disappears (so deleting the dist dir forces a rebuild).
    for rel in ["src", "index.html", "package.json", "vite.config.ts"] {
        println!(
            "cargo:rerun-if-changed={}",
            frontend_dir.join(rel).display()
        );
    }
    println!(
        "cargo:rerun-if-changed={}",
        real_dist.join("index.html").display()
    );

    // If frontend/dist/index.html already exists (pre-built or prior npm run
    // build), use it directly and skip the npm step.
    if real_dist.join("index.html").exists() {
        println!("cargo:rustc-env=SDLC_FRONTEND_DIST={}", real_dist.display());
        return;
    }

    // No pre-built dist — attempt to build the frontend now.
    // Requires Node.js ≥ 18: https://nodejs.org
    //
    // Skip npm entirely when SDLC_NO_NPM=1 (e.g. `cargo test --all` in CI or
    // development where the server UI is not needed). The stub index.html is
    // used in that case. Pre-build with `cd frontend && npm ci && npm run build`
    // when you need the real UI.
    println!("cargo:rerun-if-env-changed=SDLC_NO_NPM");
    if std::env::var("SDLC_NO_NPM").is_ok() {
        let stub_dir = out_dir.join("frontend-dist");
        std::fs::create_dir_all(&stub_dir).expect("create stub frontend-dist dir");
        std::fs::write(
            stub_dir.join("index.html"),
            "<!doctype html><html><body>\
             <p>Frontend not built (SDLC_NO_NPM set). Run <code>npm ci &amp;&amp; npm run build</code> \
             inside the <code>frontend/</code> directory.</p>\
             </body></html>",
        )
        .expect("write stub index.html");
        println!("cargo:rustc-env=SDLC_FRONTEND_DIST={}", stub_dir.display());
        println!("cargo:warning=SDLC_NO_NPM set — sdlc-server will serve a stub UI");
        return;
    }

    let npm = if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    };

    let built = Command::new(npm)
        .args(["ci"])
        .current_dir(&frontend_dir)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
        && Command::new(npm)
            .args(["run", "build"])
            .current_dir(&frontend_dir)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

    if built {
        println!("cargo:rustc-env=SDLC_FRONTEND_DIST={}", real_dist.display());
    } else {
        // npm is unavailable or failed (e.g. `cargo install` without Node.js).
        // Write a stub into OUT_DIR — always writable, never touches the source
        // tree — so the RustEmbed proc-macro does not fail at compile time.
        let stub_dir = out_dir.join("frontend-dist");
        std::fs::create_dir_all(&stub_dir).expect("create stub frontend-dist dir");
        std::fs::write(
            stub_dir.join("index.html"),
            "<!doctype html><html><body>\
             <p>Frontend not built. Run <code>npm ci &amp;&amp; npm run build</code> \
             inside the <code>frontend/</code> directory.</p>\
             </body></html>",
        )
        .expect("write stub index.html");
        println!("cargo:rustc-env=SDLC_FRONTEND_DIST={}", stub_dir.display());
        println!(
            "cargo:warning=frontend/dist not found and npm build skipped — \
             sdlc-server will serve a stub UI"
        );
    }
}
