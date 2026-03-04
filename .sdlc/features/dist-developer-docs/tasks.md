# Tasks: DEVELOPER.md Cross-Platform Dev Install Path

## T1 — Audit current DEVELOPER.md

Review the current `DEVELOPER.md` against all acceptance criteria in the spec:
- Confirm no `make` references exist
- Confirm `just` install line shows all three managers (cargo / brew / winget)
- Confirm Windows dev loop variant is present
- Confirm just recipe table is accurate
- Identify any gaps

## T2 — Add "Without just" fallback section

Add a clearly marked fallback subsection under "Install" for contributors who prefer not to install `just`. The section must show:
1. Build the frontend: `cd frontend && npm ci && npm run build`
2. Install the binary: `cargo install --path crates/sdlc-cli --locked`
3. Create the `sdlc` alias (Unix `ln -sf`, or note for Windows)

## T3 — Verify and polish existing content

After adding the fallback section:
- Verify the `just install` command line is correct against the current Justfile
- Verify all recipes in the "Other recipes" table exist in the Justfile
- Ensure the Windows dev loop variant is present and accurate
- Fix any formatting inconsistencies

## T4 — Final make-grep check

Run a final check to confirm `make` does not appear anywhere in `DEVELOPER.md`.
