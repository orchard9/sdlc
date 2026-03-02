// Library entry point for sdlc-cli.
//
// This crate is primarily a binary (`sdlc`). This thin lib target exists so
// that integration tests in `tests/` can call `pub` functions from `cmd/`
// directly, without going through the binary subprocess.
//
// Only the subset of modules needed by integration tests is declared here.
// The full command set remains in `main.rs`.

pub mod cmd;
pub mod output;
pub mod root;
pub mod tools;
