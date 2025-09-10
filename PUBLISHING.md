# Publishing Riglr to crates.io

This guide walks you through publishing the Riglr crates to crates.io and setting up automated documentation on docs.rs.

## Quick Start

```bash
# 1. Verify everything is ready
./scripts/verify-publish.sh

# 2. Configure docs.rs metadata
./scripts/configure-docs-rs.sh

# 3. Update dependencies for publishing
./scripts/update-dependencies.sh --version 0.3.0

# 4. Dry run
./scripts/publish.sh --dry-run

# 5. Publish to crates.io
export CARGO_REGISTRY_TOKEN=your_token_here
./scripts/publish.sh

# 6. Restore local development setup
./scripts/update-dependencies.sh --restore
```

## Prerequisites

### 1. crates.io Account Setup
1. Create account: https://crates.io
2. Get API token: https://crates.io/me
3. Set token: `export CARGO_REGISTRY_TOKEN=your_token`

### 2. Repository Preparation
- All tests passing: `cargo test --all`
- Clean build: `cargo build --all`
- No uncommitted changes: `git status`

## Automated Publishing

### GitHub Actions
The repository includes automated publishing via GitHub Actions:

1. **Add Secret**: Go to Settings → Secrets → Actions → New secret
   - Name: `CARGO_REGISTRY_TOKEN`
   - Value: Your crates.io API token

2. **Trigger Publishing**:
   - Manual: Actions tab → "Publish to crates.io" → Run workflow
   - Automatic: Create a release or push a version tag

## Manual Publishing Steps

### 1. Pre-flight Checks
```bash
# Run the verification script
./scripts/verify-publish.sh

# This checks:
# ✓ All required Cargo.toml fields
# ✓ README files exist
# ✓ Crates build successfully
# ✓ Dependencies are correct
```

### 2. Version Management
```bash
# Update version in workspace Cargo.toml
# All member crates inherit this version
version = "0.3.0"

# Commit version bump
git add -A
git commit -m "chore: bump version to 0.3.0"
```

### 3. Dependency Conversion
```bash
# Convert path dependencies to version dependencies
./scripts/update-dependencies.sh --version 0.3.0

# This changes:
# FROM: riglr-core = { path = "../riglr-core" }
# TO:   riglr-core = { version = "0.3.0" }
```

### 4. Publishing
```bash
# Always do a dry run first
./scripts/publish.sh --dry-run

# If successful, publish for real
./scripts/publish.sh

# The script will:
# - Publish crates in dependency order
# - Wait between publishes for indexing
# - Handle all 21 crates automatically
```

### 5. Post-Publishing
```bash
# Restore path dependencies for development
./scripts/update-dependencies.sh --restore

# Create git tag
git tag v0.3.0
git push origin v0.3.0

# Create GitHub release
# Go to: https://github.com/riglr/riglr/releases/new
```

## Verification

### crates.io
Check that all crates are published:
- https://crates.io/search?q=riglr

### docs.rs
Documentation builds automatically:
- https://docs.rs/riglr-core
- https://docs.rs/riglr-config
- (etc. for all crates)

## Crate Publishing Order

The scripts handle this automatically, but for reference:

```
1. Base (no deps):        riglr-config, riglr-macros
2. Core:                  riglr-core, riglr-evm-common  
3. Tools:                 riglr-solana-tools, riglr-evm-tools, etc.
4. Advanced:              riglr-auth, riglr-streams, etc.
5. High-level:            riglr-agents, riglr-server, etc.
6. Demo:                  riglr-showcase, riglr-indexer
7. CLI (optional):        create-riglr-app
```

## Troubleshooting

### Common Issues

**"Package already exists"**
- Version already published
- Solution: Bump version number

**"Failed to verify package"**
- Missing required fields
- Solution: Run `./scripts/verify-publish.sh`

**"Unresolved dependencies"**
- Dependency not yet published
- Solution: Ensure publishing order is correct

**"Rate limit exceeded"**
- Publishing too fast
- Solution: Use `--delay 60` flag

### Getting Help

1. Check scripts documentation: `./scripts/README.md`
2. View workflow logs: Actions tab in GitHub
3. Check crates.io status: https://crates.io
4. docs.rs build logs: https://docs.rs/crate/[crate-name]/builds

## Scripts Reference

| Script | Purpose | Usage |
|--------|---------|-------|
| `verify-publish.sh` | Pre-flight checks | `./scripts/verify-publish.sh` |
| `publish.sh` | Publish to crates.io | `./scripts/publish.sh [--dry-run]` |
| `update-dependencies.sh` | Convert dependencies | `./scripts/update-dependencies.sh [--restore]` |
| `configure-docs-rs.sh` | Setup docs.rs config | `./scripts/configure-docs-rs.sh` |

## Release Checklist

- [ ] Update version in workspace Cargo.toml
- [ ] Update CHANGELOG.md
- [ ] Run `cargo test --all`
- [ ] Run `./scripts/verify-publish.sh`
- [ ] Run `./scripts/configure-docs-rs.sh`
- [ ] Run `./scripts/update-dependencies.sh`
- [ ] Run `./scripts/publish.sh --dry-run`
- [ ] Set `CARGO_REGISTRY_TOKEN`
- [ ] Run `./scripts/publish.sh`
- [ ] Verify on crates.io
- [ ] Check docs.rs builds
- [ ] Run `./scripts/update-dependencies.sh --restore`
- [ ] Create git tag
- [ ] Create GitHub release
- [ ] Update website/documentation
- [ ] Announce release

## Notes

- First-time publishing will take longer (no caching)
- docs.rs typically builds within 5-30 minutes
- Versions cannot be deleted, only yanked
- Consider using release candidates (0.3.0-rc.1) for testing