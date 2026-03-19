# Security Policy

## Supported Versions

Only the latest release is supported with security updates.

| Version | Supported          |
| ------- | ------------------ |
| latest  | :white_check_mark: |
| < latest | :x:               |

## Reporting a Vulnerability

If you discover a security vulnerability in graxaim, please report it responsibly:

1. **GitHub Security Advisory** (preferred): Open a [Security Advisory](https://github.com/gabriel-taufer/graxaim/security/advisories/new) on the repository
2. **Email**: Contact the maintainer directly at the email listed on the GitHub profile

**Please do not open a public issue for security vulnerabilities.**

We will acknowledge receipt within 48 hours and aim to provide a fix or mitigation within 7 days for confirmed vulnerabilities.

## Scope

The following areas are in scope for security reports:

- **`.env` file handling** — reading, writing, and parsing of environment files
- **Hook execution** — pre/post-switch hook scripts and their sandboxing
- **Encryption** (Phase 5, planned) — value encryption at rest
- **Path traversal** — any way to read/write files outside the expected project scope

## How graxaim Handles Sensitive Data

graxaim is designed with security-conscious defaults:

- **Values are redacted by default** in all terminal output (`***` replaces actual values)
- **Passphrases** (planned for Phase 5) use `secrecy::SecretString` to prevent accidental logging or memory dumps
- **No telemetry** — graxaim collects zero usage data
- **No network calls** — graxaim is fully offline; it never phones home or contacts external services
- **Local-only storage** — all profile data stays on your filesystem
