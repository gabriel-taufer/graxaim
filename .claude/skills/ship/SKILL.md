---
name: ship
description: >
  Release workflow for graxaim. Pre-release checks, version bump, changelog
  update, git tag, and push to trigger CI.
triggers:
  - "release"
  - "ship it"
  - "version bump"
  - "publish"
  - "tag release"
---

# graxaim Ship Workflow

## Pre-Release Checklist

Run all checks before any release:

```bash
# 1. Format check
cargo fmt --all --check

# 2. Lint check (zero warnings)
cargo clippy --all-targets -- -D warnings

# 3. All tests pass
cargo test --all

# 4. Release build succeeds
cargo build --release

# 5. Binary runs
target/release/graxaim --version
```

All five must pass. Do not proceed if any step fails.

## Version Bump

### 1. Update `Cargo.toml`

```toml
[package]
name = "graxaim"
version = "0.2.0"  # bump this
```

### 2. Update `CHANGELOG.md`

Follow [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [0.2.0] - 2025-XX-XX

### Added
- New `diff` command for comparing profiles
- Schema validation support

### Fixed
- Broken symlink handling on profile delete

### Changed
- Improved error messages for invalid profile names
```

### 3. Commit the version bump

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
```

## Git Tag

Create an annotated tag:

```bash
git tag -a v0.2.0 -m "Release v0.2.0"
```

## Push

Push the commit and tag:

```bash
git push origin main
git push origin v0.2.0
```

This triggers the GitHub Actions release workflow (if configured).

## Semantic Versioning

Follow [SemVer](https://semver.org/):

| Change Type | Version Bump | Example |
|-------------|-------------|---------|
| Breaking CLI change | Major (`X.0.0`) | Rename a subcommand |
| New command / feature | Minor (`0.X.0`) | Add `graxaim diff` |
| Bug fix | Patch (`0.0.X`) | Fix symlink edge case |

While in `0.x.y` (pre-1.0), minor bumps may include breaking changes.

## Post-Release

After pushing the tag:

1. Verify CI/CD pipeline runs successfully
2. Check that release artifacts are created (if applicable)
3. Update any installation docs if needed
4. Announce the release if it includes user-facing changes
