# Contract: CI Workflow

**File**: `.github/workflows/ci.yml`
**Purpose**: Continuous integration - lint, test, build on every push and PR

## Triggers

| Event | Condition | Behavior |
|-------|-----------|----------|
| push | Any branch | Run full CI pipeline |
| pull_request | opened, synchronize, reopened | Run full CI pipeline |

## Concurrency

- **Group**: `ci-${{ github.ref }}`
- **Cancel in progress**: Yes (new pushes cancel running jobs)

## Jobs

### Job: lint

**Purpose**: Verify code formatting and lint rules

| Step | Action/Command | Purpose |
|------|----------------|---------|
| Checkout | `actions/checkout@v4` | Get source code |
| Setup Rust | `dtolnay/rust-toolchain@stable` | Install stable Rust with rustfmt, clippy |
| Cache | `Swatinem/rust-cache@v2` | Restore cached dependencies |
| Format check | `cargo fmt --all -- --check` | Verify formatting |
| Clippy | `cargo clippy --all-targets --all-features -- -D warnings` | Lint with warnings as errors |

**Exit Codes**:
- 0: All checks pass
- Non-zero: Formatting or lint errors found

### Job: test

**Purpose**: Run all tests

| Step | Action/Command | Purpose |
|------|----------------|---------|
| Checkout | `actions/checkout@v4` | Get source code |
| Setup Rust | `dtolnay/rust-toolchain@stable` | Install stable Rust |
| Cache | `Swatinem/rust-cache@v2` | Restore cached dependencies |
| Test | `cargo test --all-features` | Run all tests |

**Exit Codes**:
- 0: All tests pass
- Non-zero: One or more tests failed

### Job: build

**Purpose**: Verify release build and create artifact

| Step | Action/Command | Purpose |
|------|----------------|---------|
| Checkout | `actions/checkout@v4` | Get source code |
| Setup Rust | `dtolnay/rust-toolchain@stable` | Install stable Rust |
| Cache | `Swatinem/rust-cache@v2` | Restore cached dependencies |
| Build | `cargo build --release` | Compile release binary |
| Upload | `actions/upload-artifact@v4` | Store binary for download |

**Artifact**:
- Name: `spec-tui-linux-x86_64`
- Path: `target/release/spec-tui`
- Retention: 7 days

**Exit Codes**:
- 0: Build succeeds
- Non-zero: Compilation error

## Status Reporting

All jobs report status to GitHub:
- Commit status checks visible on commit page
- PR checks visible in PR conversation
- Required for branch protection rules

## Performance Targets

| Job | Target Duration | Notes |
|-----|-----------------|-------|
| lint | < 2 min | Minimal compilation needed |
| test | < 4 min | Full test suite |
| build | < 4 min | Release optimization |
| Total | < 5 min | Jobs run in parallel |

## Error Handling

- **Network errors**: GitHub retries internally
- **Timeout**: Default 6 hours (override if needed)
- **OOM**: Rare on ubuntu-latest (7GB RAM)
