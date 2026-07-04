# Development Guide for Hello Euclid

This document describes the development setup, quality checks, and build workflow.

## Code Quality

### Linting with Clippy

We use Clippy with strict pedantic checks enabled to catch common mistakes and idioms.

```bash
# Check all targets (lib, tests, benches, examples)
cargo clippy --all-targets -- -D warnings -W clippy::pedantic

# Or use the convenience script
./check.sh
```

**Lint Configuration:**

- `.vscode/settings.json` — rust-analyzer integration with pedantic warnings
- `src/lib.rs` — Crate-level lint attributes (`#![warn(...)]`)

### Formatting

We use `rustfmt` to enforce consistent code style.

```bash
# Check formatting
cargo fmt --check

# Auto-fix formatting
cargo fmt
```

### Security & Dependency Management

We provide configurations for optional dependency scanning tools:

#### cargo-audit (Security Vulnerabilities)

Install: `cargo install cargo-audit`

```bash
cargo audit
```

Checks dependencies against the [RustSec Advisory Database](https://rustsec.org/).

#### cargo-deny (Licenses, Duplicates, Banned Crates)

Install: `cargo install cargo-deny`

```bash
cargo deny check
```

Configuration: `deny.toml`

- Allowed licenses: Apache-2.0, MIT, ISC, BSD-2/3-Clause
- Detects: duplicate versions, multiple versions of the same crate, banned crates
- Checks git sources for known security issues

#### cargo-machete (Unused Dependencies)

Install: `cargo install cargo-machete`

```bash
cargo machete
```

### Complete Quality Check

Run all checks together:

```bash
./check.sh
```

This script runs:

1. Clippy with warnings as errors
2. `cargo fmt --check`
3. `cargo audit` (if installed)
4. `cargo deny check` (if installed)
5. `cargo machete` (optional)

## Building

### Debug Build

```bash
cargo build --lib
```

### Release Build

```bash
cargo build --lib --release
```

Optimizations: `lto = "thin"`, `strip = true`

## Testing

### Unit Tests

```bash
# Run all tests
cargo test --lib

# Run specific test
cargo test --lib euclid::tests::test_e_3_8

# Run with output
cargo test --lib -- --nocapture
```

Euclidean algorithm tests verify:

- Canonical patterns (E(3,8), E(5,8), E(2,5), E(7,16))
- Rotation logic
- Edge cases (empty, zero pulses, all pulses)

## Debugging

### LLDB Debugging (Standalone)

Launch configuration in `.vscode/launch.json`:

```bash
Ctrl+F5 (or Debug > Start Debugging)
```

This builds and launches the standalone binary with LLDB attached.

### In-Editor Rust Analysis

VS Code rust-analyzer provides:

- Inline type hints
- Hover documentation
- Quick fixes from Clippy
- Code completion

## Project Structure

```text
hello-euclid/
├── src/
│   ├── lib.rs          # Plugin trait impl, param definitions
│   ├── euclid.rs       # Bjorklund algorithm (pure, testable)
│   ├── sequencer.rs    # Sequencer state (Phase 3)
│   ├── standalone.rs   # Standalone binary stub
│   └── editor/
│       ├── mod.rs      # egui editor setup (Phase 4)
│       └── canvas.rs   # Circular visualization (Phase 4)
├── xtask/
│   └── src/main.rs     # Build bundler
├── Cargo.toml          # Package manifest + lint config
├── deny.toml           # Dependency security policy
├── clippy.toml         # Clippy settings
├── check.sh            # Quality check script
└── specification.md    # Feature specification

```

## CI/CD

Recommended CI checks (not yet configured):

```yaml
# Example GitHub Actions
- cargo fmt --check
- cargo clippy --all-targets -- -D warnings -W clippy::pedantic
- cargo test --lib
- cargo audit
- cargo deny check
```

## Common Issues

### "Error: no such command: `audit`/`deny`"

These are optional tools. Install with:

```bash
cargo install cargo-audit
cargo install cargo-deny
```

Or remove them from `check.sh` if not needed.

### "warning: code that will be rejected by a future version of Rust"

This is typically from transitive dependencies (like `block` crate). Monitor for updates but not critical for now.

### Clippy errors in VS Code after code changes

rust-analyzer cache may be stale. Run:

```bash
cargo clean
cargo build --lib
```

## Next Steps

- **Phase 2:** MIDI IO layout smoke test in Cubase
- **Phase 3:** Sequencer logic and note scheduling
- **Phase 4:** GUI with egui and circular canvas
- **Phase 5:** Integration testing and cross-platform builds

For details, see [specification.md](specification.md).
