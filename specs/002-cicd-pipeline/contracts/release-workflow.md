# Contract: Release Workflow

**File**: `.github/workflows/release.yml`
**Purpose**: Automated release creation when version tags are pushed

## Triggers

| Event | Condition | Behavior |
|-------|-----------|----------|
| push.tags | `v*.*.*` pattern | Build and create GitHub release |

**Tag Pattern Examples**:
- `v1.0.0` - Valid
- `v2.1.3` - Valid
- `v0.1.0-beta` - Not matched (no pre-release support)
- `1.0.0` - Not matched (missing 'v' prefix)

## Permissions

```yaml
permissions:
  contents: write  # Required to create releases and upload assets
```

## Jobs

### Job: release

**Purpose**: Build release binary and publish to GitHub Releases

| Step | Action/Command | Purpose |
|------|----------------|---------|
| Checkout | `actions/checkout@v4` | Get source code |
| Setup Rust | `dtolnay/rust-toolchain@stable` | Install stable Rust |
| Cache | `Swatinem/rust-cache@v2` | Restore cached dependencies |
| Build | `cargo build --release` | Compile optimized binary |
| Package | `tar -czvf ...` | Create distributable archive |
| Release | `softprops/action-gh-release@v1` | Create GitHub release with asset |

## Artifact Details

**Archive Name**: `speck-{version}-linux-x86_64.tar.gz`

**Archive Contents**:
```
speck-v1.0.0-linux-x86_64/
├── speck          # Binary executable
├── README.md         # Usage instructions (from repo)
└── LICENSE           # License file (from repo)
```

**Version Extraction**: `${{ github.ref_name }}` (e.g., `v1.0.0`)

## Release Configuration

| Setting | Value | Rationale |
|---------|-------|-----------|
| draft | false | Publish immediately |
| prerelease | false | Only stable releases via this workflow |
| generate_release_notes | true | Auto-generate from commits since last tag |

## Release Notes

Auto-generated from commits between tags. Format:

```markdown
## What's Changed
* feat: Add new feature by @user in #123
* fix: Fix bug by @user in #124

**Full Changelog**: https://github.com/owner/repo/compare/v0.9.0...v1.0.0
```

## Error Handling

| Failure Type | Behavior |
|--------------|----------|
| Build failure | Workflow fails, no release created |
| Package failure | Workflow fails, no release created |
| Release API error | Workflow fails, may need manual cleanup |

## Rollback Procedure

If a release needs to be removed:
1. Delete the release from GitHub Releases page
2. Delete the tag: `git push --delete origin v1.0.0`
3. Delete local tag: `git tag -d v1.0.0`

## Performance Targets

| Step | Target Duration | Notes |
|------|-----------------|-------|
| Build | < 5 min | Release mode optimization |
| Package | < 10 sec | Tar compression |
| Upload | < 30 sec | Single binary ~5MB |
| Total | < 6 min | Sequential steps |
