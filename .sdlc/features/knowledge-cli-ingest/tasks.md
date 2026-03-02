# Tasks: knowledge-cli-ingest

- [x] [user-gap] sdlc knowledge status: show init state, entry count, catalog size, last-maintained date
- [x] [user-gap] --from-url v10 behavior: fetch page title+meta via HTTP, store as source
- [x] [user-gap] empty-state messaging on list/search when not initialized
- [x] [user-gap] sdlc knowledge list includes summary column
- [ ] Add ureq dependency to sdlc-cli/Cargo.toml
- [ ] Add `pub mod knowledge;` to crates/sdlc-cli/src/cmd/mod.rs
- [ ] Create crates/sdlc-cli/src/cmd/knowledge.rs — full CLI module
- [ ] Add Knowledge subcommand + handler to crates/sdlc-cli/src/main.rs
- [ ] Add `pub mod knowledge;` to crates/sdlc-server/src/routes/mod.rs
- [ ] Create crates/sdlc-server/src/routes/knowledge.rs — REST routes
- [ ] Register knowledge routes in crates/sdlc-server/src/lib.rs
- [ ] Verify: SDLC_NO_NPM=1 cargo test --all passes
- [ ] Verify: cargo clippy --all -- -D warnings passes
