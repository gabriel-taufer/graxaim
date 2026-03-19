# Verify Build

Run the full quality check for graxaim. Execute each step in order and report pass/fail for each.

## Steps

1. **Format check**
   ```bash
   cargo fmt --all --check
   ```

2. **Lint check** (zero warnings required)
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

3. **Run all tests**
   ```bash
   cargo test --all
   ```

4. **Release build**
   ```bash
   cargo build --release
   ```

5. **Version check** (verify binary runs)
   ```bash
   target/release/graxaim --version
   ```

## Report Format

After running all steps, report results like:

```
✅ cargo fmt       — passed
✅ cargo clippy    — passed
✅ cargo test      — passed (X tests)
✅ cargo build     — passed
✅ graxaim version — vX.Y.Z
```

If any step fails, stop and report the failure with the error output.
