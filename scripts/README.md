# Riglr Publishing Scripts

This directory contains scripts for publishing the Riglr crates to crates.io and managing the release process.

## Prerequisites

1. **Rust and Cargo**: Ensure you have Rust installed
2. **crates.io Account**: Create an account at [crates.io](https://crates.io)
3. **API Token**: Get your API token from [crates.io/me](https://crates.io/me)
4. **Set Token**: Export your token: `export CARGO_REGISTRY_TOKEN=your_token_here`

## Scripts Overview

### 🔍 verify-publish.sh
Verifies that all crates are ready for publishing.

```bash
./scripts/verify-publish.sh
```

This script checks:
- Required Cargo.toml fields (name, version, description, license)
- README.md files exist and have content
- All crates build successfully
- No missing dependencies
- Which crates are already published

### 📦 publish.sh
Publishes all crates to crates.io in the correct dependency order.

```bash
# Dry run (recommended first)
./scripts/publish.sh --dry-run

# Actual publish
./scripts/publish.sh

# With custom delay between publishes
./scripts/publish.sh --delay 60

# Skip verification
./scripts/publish.sh --skip-verify
```

Options:
- `--dry-run`: Simulate publishing without actually uploading
- `--skip-verify`: Skip running tests before publishing
- `--force`: Force publish even if version already exists
- `--delay SECS`: Seconds to wait between publishes (default: 30)

### 🔄 update-dependencies.sh
Converts path dependencies to version dependencies for publishing.

```bash
# Prepare for publishing (path -> version)
./scripts/update-dependencies.sh

# Use specific version
./scripts/update-dependencies.sh --version 0.3.0

# Restore original path dependencies
./scripts/update-dependencies.sh --restore
```

### 📚 configure-docs-rs.sh
Adds docs.rs metadata configuration to ensure proper documentation building.

```bash
./scripts/configure-docs-rs.sh
```

This script:
- Adds `[package.metadata.docs.rs]` sections
- Configures feature flags for documentation
- Updates documentation URLs to point to docs.rs
- Adds conditional compilation attributes

## Publishing Workflow

### Step 1: Prepare

1. **Update Version Numbers**: 
   ```bash
   # Update version in workspace Cargo.toml
   # This will be inherited by all member crates
   ```

2. **Run Tests**:
   ```bash
   cargo test --all
   cargo clippy --all
   ```

3. **Verify Everything is Ready**:
   ```bash
   ./scripts/verify-publish.sh
   ```

### Step 2: Configure

1. **Add docs.rs Metadata**:
   ```bash
   ./scripts/configure-docs-rs.sh
   ```

2. **Update Dependencies** (for publishing):
   ```bash
   ./scripts/update-dependencies.sh --version 0.3.0
   ```

### Step 3: Publish

1. **Dry Run First**:
   ```bash
   ./scripts/publish.sh --dry-run
   ```

2. **Actual Publish**:
   ```bash
   export CARGO_REGISTRY_TOKEN=your_token_here
   ./scripts/publish.sh
   ```

3. **Monitor Progress**:
   - The script will publish crates in dependency order
   - Each tier must complete before the next begins
   - Check [crates.io/search?q=riglr](https://crates.io/search?q=riglr)

### Step 4: Post-Publish

1. **Restore Path Dependencies**:
   ```bash
   ./scripts/update-dependencies.sh --restore
   ```

2. **Verify Documentation**:
   - Check [docs.rs/riglr-core](https://docs.rs/riglr-core)
   - Documentation builds automatically after publishing

3. **Create GitHub Release**:
   ```bash
   git tag v0.3.0
   git push origin v0.3.0
   ```

## GitHub Actions

The `.github/workflows/publish.yml` workflow automates publishing:

### Manual Trigger
```yaml
# Go to Actions tab → Publish to crates.io → Run workflow
```

### Automatic Triggers
- On release creation
- On version tags (v0.3.0)

### Required Secrets
Add `CARGO_REGISTRY_TOKEN` to repository secrets:
1. Go to Settings → Secrets → Actions
2. Add new secret: `CARGO_REGISTRY_TOKEN`
3. Value: Your crates.io API token

## Dependency Order

Crates must be published in this order due to interdependencies:

```
Tier 1: riglr-config, riglr-macros
   ↓
Tier 2: riglr-core, riglr-evm-common
   ↓
Tier 3: riglr-events-core, riglr-solana-tools, riglr-evm-tools, 
        riglr-web-tools, riglr-web-adapters
   ↓
Tier 4: riglr-solana-events, riglr-hyperliquid-tools, riglr-auth,
        riglr-streams, riglr-cross-chain-tools
   ↓
Tier 5: riglr-agents, riglr-server, riglr-graph-memory
   ↓
Tier 6: riglr-showcase, riglr-indexer
   ↓
Tier 7: create-riglr-app (optional CLI)
```

## Troubleshooting

### "Package already exists"
- The version already exists on crates.io
- Bump the version number in Cargo.toml
- Versions cannot be overwritten (only yanked)

### "Failed to verify package"
- Run `./scripts/verify-publish.sh` to identify issues
- Common issues:
  - Missing required fields in Cargo.toml
  - Path dependencies not converted
  - Build failures

### "Rate limit exceeded"
- crates.io may rate limit rapid publishes
- Increase delay: `--delay 60`
- Wait a few minutes and retry

### "Documentation not building"
- Check [docs.rs/crate/riglr-core/builds](https://docs.rs/crate/riglr-core/builds)
- View build logs for errors
- Ensure docs.rs metadata is correct

## Best Practices

1. **Always Dry Run First**: Use `--dry-run` to catch issues
2. **Version Carefully**: Follow [SemVer](https://semver.org/)
3. **Document Changes**: Update CHANGELOG.md
4. **Test Thoroughly**: Run full test suite before publishing
5. **Tag Releases**: Create git tags for each version
6. **Monitor docs.rs**: Ensure documentation builds successfully

## Support

For issues or questions:
- Open an issue on [GitHub](https://github.com/riglr/riglr/issues)
- Check existing published crates at [crates.io/search?q=riglr](https://crates.io/search?q=riglr)
- View documentation at [docs.rs](https://docs.rs/riglr-core)