# Tasks: Milestone Detail — MilestonePreparePanel Integration

## T1: Add MilestonePreparePanel to MilestoneDetail page
- Add import for `MilestonePreparePanel` from `@/components/milestones/MilestonePreparePanel`
- Render `<MilestonePreparePanel milestoneSlug={slug} />` between the header and Features section
- Verify the component renders correctly with wave data and hides when no data

## T2: Build verification
- Run `SDLC_NO_NPM=1 cargo test --all` to confirm no regressions
- Run `cargo clippy --all -- -D warnings` to confirm no lint issues
