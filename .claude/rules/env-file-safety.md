# .env File Safety Rules

## Never Do

1. **Never print values to stdout by default** — Always redact unless `--no-redact` is explicitly passed.
   The `src/ui/redact.rs` module handles redaction (`sk_test_...5678` format).

2. **Never log .env values** — Not in debug output, not in error messages, not in tracing spans.

3. **Never include values in error context strings** — Say `"failed to parse KEY"` not
   `"failed to parse KEY=secret123"`.

   ```rust
   // Good
   .with_context(|| format!("failed to parse key '{}'", key))?;

   // Bad — leaks secret value
   .with_context(|| format!("failed to parse '{}={}'", key, value))?;
   ```

4. **Never write .env values to temp files** — All operations should be in-memory.
   Use `String` buffers, not intermediate files on disk.

5. **Never store passphrases in plain text** — Use `secrecy::SecretString` for any
   encryption passphrases or sensitive tokens held in memory.

## Always Do

1. **Redact by default** — Display values in `sk_test_...5678` format (first 8 chars + `...` + last 4 chars).
   Full values only when `--no-redact` is explicitly passed.

2. **Validate profile names** — `[a-zA-Z0-9_-]` only, via `ProfileName` newtype.
   Reject anything else at the CLI boundary before it reaches core logic.

3. **Add to .gitignore** — On `graxaim init`, add `.env` and `.env.*` patterns
   (except `.env.*.sealed` if encryption is enabled).

4. **Preserve file format** — Round-trip must not alter comments, blank lines, quoting style,
   or line ending format. Read what's there, write back exactly what was there plus changes.

## Edge Cases

These must all be handled correctly by `src/core/env_file.rs`:

| Case | Input | Expected Behavior |
|------|-------|-------------------|
| Empty value | `KEY=` | Store as empty string `""` |
| Quoted empty | `KEY=""` | Store as empty string `""` (preserve quotes on write) |
| Missing key | (no `KEY` line) | Return `None`, not empty string |
| Value with `=` | `BASE64=abc=def==` | Split only on first `=`: key=`BASE64`, value=`abc=def==` |
| Single quotes | `KEY='hello world'` | Preserve single quotes on round-trip |
| Double quotes | `KEY="hello world"` | Preserve double quotes on round-trip |
| Escape sequences | `KEY="line1\nline2"` | Handle `\n`, `\t`, `\\` inside double quotes |
| Windows line endings | `KEY=value\r\n` | Handle `\r\n` transparently, preserve on write |
| No trailing newline | `KEY=value` (EOF) | Don't add trailing newline if not present |
| BOM markers | `\xEF\xBB\xBFKEY=value` | Handle UTF-8 BOM gracefully (strip on parse) |
| Comments | `# this is a comment` | Preserve comments in output |
| Inline comments | `KEY=value # comment` | Treat `# comment` as part of the value (no inline comments) |
