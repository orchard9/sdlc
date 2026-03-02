# QA Results: Add File Paths to Init Error Messages

## TC1: Build passes

```
SDLC_NO_NPM=1 cargo build --all
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

PASS

## TC2: Clippy clean

```
cargo clippy --all -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

PASS — zero warnings.

## TC3: All tests pass

```
SDLC_NO_NPM=1 cargo test --all
... all test suites passed ...
test result: ok.
```

PASS — all tests green across all crates.

## TC4: Context strings present in source

Verified in `crates/sdlc-cli/src/cmd/init/mod.rs`:

| Line | Call | Context |
|---|---|---|
| 53 | `io::ensure_dir(&p)` | `"failed to create {}", p.display()` |
| 60–61 | `cfg.save(root)` | `"failed to write {}", config_path.display()` |
| 71–72 | `state.save(root)` | `"failed to write {}", state_path.display()` |
| 96 | `io::ensure_dir(&p)` | `"failed to create {}", p.display()` |
| 100–101 | `io::write_if_missing(&index_path, ...)` | `"failed to write {}", index_path.display()` |

PASS

## TC5: No remaining bare `?` on filesystem calls in init path

```
grep -n 'io::\(ensure_dir\|write_if_missing\|atomic_write\)[^;]*?;' crates/sdlc-cli/src/cmd/init/mod.rs
(no output)
```

PASS — no bare `?` on filesystem calls in the init path.

## Overall Result

All 5 test cases PASS. Ready to merge.
