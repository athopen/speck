# Research: CI/CD Pipeline with GitHub Actions

**Feature**: 002-cicd-pipeline
**Date**: 2026-01-20

## Overview

Research findings for implementing a GitHub Actions CI/CD pipeline for a Rust project, covering workflow structure, caching strategies, and release automation.

## Decision 1: Workflow File Organization

**Decision**: Use two separate workflow files (`ci.yml` and `release.yml`)

**Rationale**:
- Separation of concerns: CI runs on every push/PR, releases only on tags
- Easier to maintain and debug independently
- Different trigger conditions and job requirements
- Follows GitHub Actions community conventions

**Alternatives Considered**:
- Single workflow with conditional jobs: Rejected due to complexity and harder debugging
- Three+ workflows (lint, test, build separate): Rejected as over-engineered for this scope

## Decision 2: Rust Toolchain Action

**Decision**: Use `dtolnay/rust-toolchain` action

**Rationale**:
- Most widely adopted Rust toolchain action in the ecosystem
- Supports stable, beta, nightly, and specific version pinning
- Includes component installation (rustfmt, clippy) in single step
- Active maintenance and community support
- Faster than `actions-rs/toolchain` (deprecated)

**Alternatives Considered**:
- `actions-rs/toolchain`: Deprecated, no longer maintained
- Manual `rustup` commands: More verbose, harder to maintain
- `hecrj/setup-rust-action`: Less feature-rich

## Decision 3: Dependency Caching Strategy

**Decision**: Use `Swatinem/rust-cache` action with default settings

**Rationale**:
- Purpose-built for Rust projects, understands Cargo.lock
- Caches `~/.cargo` registry, git dependencies, and target directory
- Automatic cache key generation based on Cargo.lock hash
- Handles cache invalidation correctly on dependency changes
- Typical 50-70% build time reduction on cache hits

**Alternatives Considered**:
- `actions/cache` with manual paths: Works but requires manual key management
- No caching: Unacceptable for 10-minute target (cold builds take 5+ minutes)

## Decision 4: Concurrency Control

**Decision**: Use `concurrency` key with `cancel-in-progress: true`

**Rationale**:
- Cancels outdated runs when new commits pushed to same branch
- Saves CI minutes and provides faster feedback
- Groups by `github.workflow` + `github.ref` for proper isolation
- Standard practice for active development workflows

**Configuration**:
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

## Decision 5: Job Structure for CI Workflow

**Decision**: Three parallel jobs: `lint`, `test`, `build`

**Rationale**:
- Parallel execution minimizes total pipeline time
- Independent failures allow partial feedback (e.g., tests pass but lint fails)
- Clear separation makes logs easier to navigate
- Each job has single responsibility

**Job Details**:
| Job | Purpose | Commands |
|-----|---------|----------|
| lint | Code quality | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` |
| test | Correctness | `cargo test --all-features` |
| build | Compilation | `cargo build --release`, upload artifact |

## Decision 6: Release Workflow Trigger

**Decision**: Trigger on tags matching `v*.*.*` pattern

**Rationale**:
- Semantic versioning is industry standard
- Tag-based releases are explicit and auditable
- Separates release intent from regular development
- Easy to integrate with `git tag -a v1.0.0 -m "Release 1.0.0"`

**Pattern**: `v[0-9]+.[0-9]+.[0-9]+` (e.g., v1.0.0, v2.1.3)

## Decision 7: Release Asset Creation

**Decision**: Use `softprops/action-gh-release` action

**Rationale**:
- Most popular GitHub release action (10k+ stars)
- Supports draft releases, asset uploads, release notes
- Simple configuration, well-documented
- Handles GitHub API authentication automatically

**Alternatives Considered**:
- `actions/create-release` + `actions/upload-release-asset`: Deprecated
- GitHub CLI (`gh release create`): More verbose in workflow
- Manual API calls: Unnecessary complexity

## Decision 8: Artifact Naming Convention

**Decision**: `spec-tui-{version}-linux-x86_64.tar.gz`

**Rationale**:
- Includes tool name for identification
- Version from tag for traceability
- Platform and architecture for clarity
- `.tar.gz` is standard for Linux binaries
- Consistent with Rust ecosystem conventions (ripgrep, bat, etc.)

## Decision 9: Retry Strategy

**Decision**: No automatic retries; rely on manual re-run

**Rationale**:
- GitHub Actions UI provides one-click re-run
- Automatic retries can mask real issues
- Infrastructure failures are rare on GitHub-hosted runners
- Keeps workflow simple and debuggable

**Alternatives Considered**:
- `nick-invision/retry` action: Adds complexity, rarely needed
- Workflow-level retry: Not supported natively

## Decision 10: Branch Protection (Documentation Only)

**Decision**: Document required branch protection settings; don't automate

**Rationale**:
- Branch protection is a repository setting, not workflow
- Requires admin access to configure
- One-time setup, not part of workflow files
- Document in quickstart.md for manual configuration

**Required Settings**:
- Require status checks to pass before merging
- Required checks: `lint`, `test`, `build`
- Require branches to be up to date before merging

## Performance Targets

| Metric | Target | Strategy |
|--------|--------|----------|
| Cold build | < 8 min | Parallel jobs, optimized checkout |
| Cached build | < 4 min | Swatinem/rust-cache |
| Lint only | < 2 min | Minimal dependencies needed |
| Test only | < 3 min | Cached target directory |
| Release build | < 10 min | Release mode + artifact upload |

## Security Considerations

- Use `${{ secrets.GITHUB_TOKEN }}` for release publishing (automatic)
- No external secrets required for basic CI/CD
- Pin action versions to SHA for security (`@v4` is acceptable for trusted actions)
- Minimize permissions with `permissions:` key where possible
