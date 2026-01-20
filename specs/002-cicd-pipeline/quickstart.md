# Quickstart: CI/CD Pipeline Setup

## Prerequisites

- GitHub repository with Rust project
- `Cargo.toml` at repository root
- Git installed locally

## Setup

### 1. Create Workflow Directory

```bash
mkdir -p .github/workflows
```

### 2. Create CI Workflow

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: ['**']
  pull_request:
    types: [opened, synchronize, reopened]

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

jobs:
  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: Check formatting
        run: cargo fmt --all -- --check
      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run tests
        run: cargo test --all-features
```

### 3. Create Release Workflow

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'

permissions:
  contents: write

jobs:
  release:
    name: Release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Build release
        run: cargo build --release

      - name: Package
        run: |
          mkdir -p dist
          cp target/release/speck dist/
          cp README.md LICENSE dist/ 2>/dev/null || true
          cd dist && tar -czvf ../speck-${{ github.ref_name }}-linux-x86_64.tar.gz *

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: speck-${{ github.ref_name }}-linux-x86_64.tar.gz
          generate_release_notes: true
```

### 4. Commit and Push

```bash
git add .github/workflows/
git commit -m "Add CI/CD workflows"
git push
```

### 5. Configure Branch Protection (Optional but Recommended)

1. Go to repository Settings → Branches
2. Add branch protection rule for `master` (or `main`)
3. Enable "Require status checks to pass before merging"
4. Select required checks: `Lint`, `Test`
5. Enable "Require branches to be up to date before merging"

## Usage

### Automatic CI

Every push and pull request automatically triggers:
- Code formatting check (`cargo fmt --check`)
- Linting (`cargo clippy`)
- Tests (`cargo test`)

View results in the Actions tab or on PR/commit pages.

### Creating a Release

```bash
# Create and push a version tag
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

This triggers the release workflow which:
1. Builds the release binary
2. Creates a `.tar.gz` archive
3. Publishes a GitHub Release with the archive attached

### Downloading Artifacts

**From Releases**: Releases page → Select version → Assets → Download

## Troubleshooting

### CI not triggering

- Verify workflow files are in `.github/workflows/`
- Check file extension is `.yml` or `.yaml`
- Ensure YAML syntax is valid

### Cache not working

- First run always misses cache (expected)
- Check `Swatinem/rust-cache` logs for details
- Cache is per-branch; new branches start cold

### Release not created

- Verify tag matches pattern `v*.*.*`
- Check workflow has `contents: write` permission
- Review Actions logs for specific error

### Formatting check fails

Fix locally:
```bash
cargo fmt --all
git add -A
git commit -m "Fix formatting"
git push
```

### Clippy warnings

Fix locally:
```bash
cargo clippy --all-targets --all-features -- -D warnings
# Fix reported issues
git add -A
git commit -m "Fix clippy warnings"
git push
```

## Performance Tips

1. **Cache is key**: Subsequent runs are ~50% faster due to dependency caching
2. **Parallel jobs**: lint and test run simultaneously
3. **Cancel redundant runs**: Rapid pushes cancel outdated runs automatically
