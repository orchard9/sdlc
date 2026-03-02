# Tasks: Document sdlc update as Update Mechanism

## T1: Add "Updating" section to README.md

Add an "Updating" section immediately after the Install section in `README.md` explaining how to upgrade the binary and run `sdlc update` to sync AI command scaffolding.

## T2: Fix `sdlc init` completion message

In `crates/sdlc-cli/src/cmd/init/mod.rs`, change the "Next:" completion message from `sdlc feature create <slug>` to `sdlc ui` with instruction to visit `/setup` first.
