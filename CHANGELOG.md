# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-19

### Added
- Core profile management: `init`, `use`, `list`, `create`, `delete`, `rename`, `current`, `edit`
- `run` command — execute commands with profile environment loaded
- `export` command — generate shell-specific export commands (bash/zsh/fish)
- Interactive fuzzy profile picker (via `skim`)
- Hook system — pre/post-switch hooks with timeout and strict mode
- direnv integration — auto-generates `.envrc`
- Value redaction in output by default
- `.gitignore` auto-management on `init`
