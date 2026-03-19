# graxaim — Product Requirements Document

**Version:** 0.1 (Draft)
**Author:** [Your Name]
**Date:** February 2026

---

## About the name

**graxaim** (pronounced "gra-sha-EEM") is the pampas fox from southern Brazil (*Lycalopex gymnocercus*) — a small, clever canid known for its adaptability and resourcefulness. Native to the grasslands and pampas of Rio Grande do Sul, Uruguay, and Argentina, the graxaim thrives in diverse environments, from rural farmlands to urban edges, always finding a way to survive and prosper.

This adaptability makes it the perfect metaphor for a tool that manages development environments: smart, versatile, and comfortable switching between any context. Just as the graxaim navigates different terrains with ease, **graxaim the tool** helps developers seamlessly switch between local, staging, production, and countless other configurations without friction.

---

## 1. Problem Statement

Modern applications rely heavily on environment variables for configuration. As projects grow, developers accumulate multiple `.env` files for different contexts: local development, staging, production, per-client configurations, feature branches, and more. The current tooling landscape handles *loading* environment variables well (direnv, dotenv) and *storing secrets* well (Infisical, Vault, SOPS), but no tool owns the **profile lifecycle** — the day-to-day workflow of switching between named configurations, keeping them consistent, and catching mistakes before they cause runtime failures.

### What developers actually do today

- Manually copy-paste `.env` files and rename them (`.env.local`, `.env.staging`, `.env.client-acme`)
- Forget which profile is active, deploy with the wrong config
- Add a new required variable to one profile but forget to add it to others
- Accumulate dead variables that nothing in the codebase references anymore
- Share `.env` files over Slack or email with no encryption
- Have no visibility into what actually *changed* between two profiles

These are not secrets-management problems. They are **configuration management problems at the developer workstation level**, and they are unsolved.

---

## 2. Product Vision

**graxaim** is a local-first, zero-infrastructure CLI tool that manages named environment profiles per project. It makes switching between configurations instant, keeps profiles consistent through schema validation, and integrates seamlessly with existing tools like direnv.

### One-liner

> direnv tells your shell *how* to load env vars. graxaim tells you *which* env vars to load, and makes sure they're correct.

### Design Principles

1. **Local-first, no server required.** Everything runs on the developer's machine. No accounts, no cloud, no SaaS.
2. **Additive, not replacive.** Works alongside direnv, dotenvx, mise, and existing workflows. Doesn't try to own the shell hook.
3. **Single binary, zero config to start.** `graxaim init` in a project directory and you're running. Schema and advanced features are opt-in.
4. **Git-friendly.** Profile metadata, schemas, and sealed (encrypted) profiles are safe to commit. Plaintext profiles are `.gitignore`d by default.
5. **Fast and silent.** Written in Rust or Go. Sub-10ms for any operation. No spinners, no network calls.

---

## 3. Target Users

### Primary: Individual developers working on multi-environment projects

- Freelancers/consultants switching between client configurations
- Backend developers toggling between local, Docker, staging, production
- Full-stack developers managing separate API and frontend env configs
- Open source maintainers who need a clean `.env.example` workflow

### Secondary: Small teams (2–10 people)

- Teams that share encrypted profiles via git instead of using a secrets platform
- Teams that want to enforce a schema for required variables across all environments

### Non-target (explicitly out of scope for v1)

- Enterprise secrets management (use Vault, Infisical, Phase)
- CI/CD secret injection (use platform-native solutions)
- Runtime config management (use feature flags, remote config)

---

## 4. Feature Specification

### 4.1 Core: Profile Management

**Profiles** are named `.env` files that live in the project root. graxaim tracks them and provides a fast switching mechanism.

#### File convention

```
project/
├── .graxaim/
│   ├── config.toml          # project-level config (active profile, settings)
│   └── schema.toml          # optional: variable schema
├── .env.local               # profile: local
├── .env.staging             # profile: staging
├── .env.production          # profile: production
├── .env.client-acme         # profile: client-acme
├── .env                     # symlink → .env.local (the active profile)
└── .envrc                   # direnv file (reads from .env)
```

#### Commands

| Command | Description |
|---------|-------------|
| `graxaim init` | Initialize graxaim in the current project. Creates `.graxaim/` directory, detects existing `.env.*` files as profiles, adds `.env` and `.graxaim/config.toml` to `.gitignore`. |
| `graxaim use [name]` | Switch the active profile. Updates the `.env` symlink to point to `.env.<name>`. If no name is provided, opens an interactive fuzzy picker (fzf-style, built-in). |
| `graxaim current` | Print the name of the currently active profile. |
| `graxaim list` | List all available profiles with a marker on the active one. Shows completeness percentage if a schema is defined. |
| `graxaim create <name>` | Create a new profile. If a schema exists, pre-populates with all required keys (values left blank or filled from defaults). If another profile exists, offers to copy from it. |
| `graxaim delete <name>` | Delete a profile (with confirmation). Refuses to delete the active profile. |
| `graxaim rename <old> <new>` | Rename a profile. Updates symlink if it was the active one. |
| `graxaim edit [name]` | Open a profile in `$EDITOR`. Defaults to the active profile. Runs validation after save if a schema exists. |

### 4.2 Diffing

Compare environment profiles to understand what's different. This is the feature most requested by developers who manage multiple configs and has no good standalone solution today.

#### Commands

| Command | Description |
|---------|-------------|
| `graxaim diff <a> <b>` | Show a colored, side-by-side diff of two profiles. Groups output into: keys only in A, keys only in B, keys with different values, keys with same values (hidden by default). |
| `graxaim diff --all` | Diff all profiles against each other in a matrix view. Highlights variables that differ across any profile. |
| `graxaim diff --redact` | (Default behavior) Redact values longer than 8 characters, showing only first 3 and last 3 chars (e.g., `sk_...4xf`). Use `--no-redact` to show full values. |

#### Output example

```
graxaim diff local staging

  Profile: local → staging

  Only in local:
    DEBUG_MODE = true

  Only in staging:
    SENTRY_DSN = htt...m/5

  Different values:
    DATABASE_URL   local: pos...5432/dev    staging: pos...5432/stg
    API_BASE_URL   local: htt...t:3000      staging: htt...pp.com
    LOG_LEVEL      local: debug             staging: warn

  Same in both: 12 variables (hidden, use --show-same)
```

### 4.3 Schema Validation

An optional `schema.toml` file that declares the contract for environment variables in the project. This is the key differentiator — no existing tool does typed, per-project env schema validation with actionable error messages.

#### Schema definition

```toml
# .graxaim/schema.toml

[meta]
description = "Backend API configuration"

[vars.DATABASE_URL]
required = true
type = "url"
description = "PostgreSQL connection string"
example = "postgres://user:pass@localhost:5432/mydb"

[vars.PORT]
required = true
type = "port"                  # integer, 1-65535
default = "3000"

[vars.LOG_LEVEL]
required = true
type = "enum"
values = ["debug", "info", "warn", "error"]
default = "info"

[vars.API_SECRET]
required = true
type = "string"
sensitive = true               # always redacted in diffs/output
min_length = 32

[vars.ENABLE_CACHE]
required = false
type = "boolean"               # true/false, 1/0, yes/no
default = "true"

[vars.REDIS_URL]
required = false
type = "url"
depends_on = "ENABLE_CACHE"    # only required if ENABLE_CACHE is truthy

[vars.SMTP_PORT]
required = false
type = "integer"
min = 1
max = 65535

[vars.ALLOWED_ORIGINS]
required = false
type = "list"                  # comma-separated values
description = "CORS allowed origins"
```

#### Supported types

| Type | Validation |
|------|-----------|
| `string` | Any value. Optional `min_length`, `max_length`, `pattern` (regex). |
| `integer` | Numeric. Optional `min`, `max`. |
| `port` | Integer between 1 and 65535. |
| `boolean` | Accepts `true`, `false`, `1`, `0`, `yes`, `no` (case-insensitive). |
| `url` | Valid URL format. Optional `schemes` (e.g., `["https", "postgres"]`). |
| `email` | Valid email format. |
| `enum` | Must be one of the specified `values`. |
| `list` | Comma-separated values. Optional `item_type` for each element. |
| `path` | File or directory path. Optional `must_exist = true`. |

#### Commands

| Command | Description |
|---------|-------------|
| `graxaim check [name]` | Validate a profile (or all profiles) against the schema. Reports: missing required vars, type mismatches, constraint violations, unknown vars not in schema. |
| `graxaim check --all` | Validate every profile and produce a summary matrix. |
| `graxaim schema init` | Generate a `schema.toml` from the current active profile by inferring types from values. Developer reviews and refines. |
| `graxaim schema generate-example` | Generate a `.env.example` file from the schema with descriptions as comments, example values, and all required keys. |

#### Validation output example

```
graxaim check staging

  Validating staging against schema...

  ✗ MISSING    API_SECRET (required, string, min_length=32)
  ✗ TYPE       PORT = "not_a_number" (expected: port, got: non-numeric string)
  ✗ ENUM       LOG_LEVEL = "verbose" (expected one of: debug, info, warn, error)
  ⚠ UNKNOWN    LEGACY_FLAG (not defined in schema)
  ✓ 14 variables passed validation

  Result: 3 errors, 1 warning
```

### 4.4 Codebase Audit

Scan the project source code to find references to environment variables and cross-reference them with profiles and schema. This catches the "I removed the feature but left the env var" and "I added a new env var in code but forgot to add it to my profiles" bugs.

#### How it works

graxaim scans source files for common patterns:

| Language | Patterns detected |
|----------|------------------|
| JavaScript/TypeScript | `process.env.VAR`, `process.env['VAR']`, `import.meta.env.VAR` |
| Python | `os.environ['VAR']`, `os.getenv('VAR')`, `os.environ.get('VAR')` |
| Rust | `env::var("VAR")`, `env!("VAR")` |
| Go | `os.Getenv("VAR")` |
| Ruby | `ENV['VAR']`, `ENV.fetch('VAR')` |
| PHP | `getenv('VAR')`, `$_ENV['VAR']` |
| Generic | `.env` file references, docker-compose.yml `${VAR}` interpolation |

#### Commands

| Command | Description |
|---------|-------------|
| `graxaim audit` | Full audit. Reports: vars in code but missing from all profiles, vars in profiles but never referenced in code, vars in schema but not in code (stale schema entries). |
| `graxaim audit --profile <name>` | Audit a specific profile against code references. |

#### Output example

```
graxaim audit

  Scanning source files... 47 files scanned

  Referenced in code but MISSING from profiles:
    STRIPE_WEBHOOK_SECRET    found in: src/payments/webhook.ts:14
    NEW_FEATURE_FLAG         found in: src/features/index.ts:3

  In profiles but NOT referenced in code:
    OLD_API_KEY              present in: local, staging, production
    DEPRECATED_URL           present in: local

  Summary: 2 missing from profiles, 2 potentially dead variables
```

### 4.5 Encryption (Seal/Unseal)

Allow profiles to be encrypted and committed to git. Uses `age` encryption for simplicity and security. This is intentionally simpler than SOPS — it encrypts the entire file, not individual values, because the use case is "I want to safely commit my staging profile to git", not "I want to edit encrypted files in place."

#### Commands

| Command | Description |
|---------|-------------|
| `graxaim seal <name>` | Encrypt a profile using `age`. Produces `.env.<name>.sealed`. Prompts for a passphrase or accepts an age public key via `--recipient`. |
| `graxaim unseal <name>` | Decrypt a `.sealed` profile back to its plaintext `.env.<name>` file. |
| `graxaim seal --all` | Seal all profiles. |

#### File convention

```
.env.production           # plaintext (gitignored)
.env.production.sealed    # encrypted (safe to commit)
```

### 4.6 direnv Integration

graxaim should work seamlessly with direnv but never require it.

#### `graxaim init` generates a `.envrc`

```bash
# .envrc (generated by graxaim)
dotenv .env
```

When the developer runs `graxaim use staging`, the `.env` symlink updates, and the next time direnv triggers (on `cd` or `direnv reload`), the new profile is loaded.

#### Without direnv

Developers who don't use direnv can use:

```bash
eval $(graxaim export)          # export active profile to current shell
graxaim run -- npm start        # run a command with the active profile injected
graxaim run -p staging -- npm start  # run with a specific profile
```

---

## 5. Non-Functional Requirements

### Performance

- All commands complete in under 50ms for projects with fewer than 20 profiles
- Codebase audit scans at least 10,000 files per second (AST-free, regex-based)
- Zero network calls. Ever. (Except for `graxaim update` self-update check, which is opt-in)

### Portability

- Single static binary for Linux (x86_64, aarch64), macOS (Intel, Apple Silicon), Windows
- No runtime dependencies (no Node.js, no Python, no Ruby)
- Works in any shell (bash, zsh, fish, PowerShell)

### Security

- Plaintext profiles are never written to stdout unless explicitly requested
- `--redact` is the default for all output that includes values
- Sealed profiles use `age` (audited, modern encryption)
- No telemetry, no analytics, no phoning home

---

## 6. What graxaim Is NOT

| Not this | Use this instead |
|----------|-----------------|
| A secrets manager for teams/orgs | Infisical, Phase, Vault |
| A shell hook / env loader | direnv, mise, dotenvx |
| A CI/CD secret injector | Platform-native (GitHub Actions secrets, AWS SSM) |
| A runtime config service | LaunchDarkly, Consul, etcd |
| A `.env` syntax linter | dotenv-linter |
| An env var encryption-in-place tool | SOPS |

graxaim is the **missing layer** between your `.env` files and the tools that load them.

---

## 7. User Journeys

### Journey 1: Solo developer starting a new project

```
$ cd my-project
$ graxaim init
  ✓ Created .graxaim/
  ✓ Detected 2 existing profiles: local, staging
  ✓ Active profile: local (.env → .env.local)
  ✓ Added .env to .gitignore
  ✓ Generated .envrc for direnv

$ graxaim create production --from staging
  ✓ Created .env.production (copied from staging)
  ✓ Open in editor? (Y/n)
```

### Journey 2: Consultant switching between clients

```
$ graxaim list
    client-acme    (active)
    client-globex
    local-dev

$ graxaim use client-globex
  ✓ Switched to: client-globex
  ℹ direnv: loading .env

$ graxaim diff client-acme client-globex
  ... (shows what's different between clients)
```

### Journey 3: Team enforcing consistency

```
$ graxaim schema init
  ✓ Generated .graxaim/schema.toml from active profile (18 variables)
  ℹ Review and adjust types, then commit to git

$ graxaim check --all
  local:       ✓ 18/18 passed
  staging:     ✗ 1 error (missing STRIPE_WEBHOOK_SECRET)
  production:  ✗ 2 errors (missing STRIPE_WEBHOOK_SECRET, invalid LOG_LEVEL)
```

### Journey 4: Onboarding a new team member

```
$ git clone <repo>
$ cd project
$ graxaim unseal local
  🔑 Enter passphrase: ****
  ✓ Decrypted .env.local

$ graxaim use local
  ✓ Switched to: local
  ✓ Ready to develop
```

---

## 8. Release Plan

### v0.1 — MVP

Core profile management: `init`, `use`, `list`, `create`, `delete`, `current`, `edit`. Interactive picker. direnv integration. `run` and `export` commands.

**Goal:** Replace the shell function. Validate the workflow.

### v0.2 — Diff & Check

Profile diffing with redaction. Schema definition and validation (`check`). `.env.example` generation from schema.

**Goal:** Prove the schema validation value proposition.

### v0.3 — Audit & Seal

Codebase audit (code-aware dead variable detection). `seal`/`unseal` with age encryption.

**Goal:** Complete the feature set for solo developers.

### v1.0 — Polish & Ecosystem

Shell completions (bash, zsh, fish). Pre-commit hook integration. VS Code extension (profile picker in status bar). Comprehensive documentation site. Homebrew / apt / cargo install.

**Goal:** Ready for public launch and community adoption.

---

## 9. Success Metrics

- **Adoption:** 500+ GitHub stars within 3 months of public launch
- **Retention:** Developers who try it keep it in their workflow (measured via community feedback)
- **Ecosystem:** At least 2 community-contributed integrations (IDE plugin, CI action) within 6 months
- **Quality:** Zero data loss bugs (profiles are precious). Zero false positives in schema validation.

---

## 10. Open Questions

1. **Naming:** `graxaim`, `envp`, `envsw`, `envprof`, or something else? The name should be short, memorable, and available on crates.io / Homebrew / npm.
2. **Language:** Rust (fastest, single binary, great CLI ecosystem with `clap`) vs Go (faster to prototype, still single binary). Leaning Rust.
3. **Schema format:** TOML (consistent with Rust ecosystem, readable) vs JSON Schema (standard, but verbose) vs YAML. Leaning TOML.
4. **Monorepo support:** Should graxaim support per-package profiles within a monorepo, or is project-root-level sufficient for v1?
5. **Profile inheritance:** Should a profile be able to `extend` another (e.g., `staging` extends `local` but overrides 3 vars)? Powerful but adds complexity.
6. **Hook system:** Should graxaim support pre/post-switch hooks (e.g., run a script after switching to `production` that warns you)?