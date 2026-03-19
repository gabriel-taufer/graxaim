# Contributing to graxaim

Thanks for your interest in contributing! This document covers how to get started.

## Building

```bash
cargo build
```

## Running Tests

```bash
cargo test
```

## Quality Gate

All PRs must pass the full quality gate before merging:

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test
```

## PR Expectations

- **Focused changes** — one logical change per PR
- **Tests required** for new features and bug fixes
- **Pass the quality gate** — CI will enforce this automatically

## Branch Naming

Follow conventional commit scoping for branch names:

- `fix(scope): description` — bug fixes
- `feat(scope): description` — new features
- `chore(scope): description` — maintenance, refactoring, docs

Examples: `feat(export): add powershell support`, `fix(hooks): timeout race condition`

## Implementation References

- See [`CLAUDE.md`](CLAUDE.md) for architecture overview and implementation details
- See [`.claude/rules/`](.claude/rules/) for coding patterns and conventions used in this project
