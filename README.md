# ghostty-xterm-compat-serialize

Ghostty-backed xterm serialize compatibility tooling.

This repository has two responsibilities:

- `crates/ghostty-xterm-compat-serialize` — a reusable Rust crate that produces xterm-compatible serialize output from `libghostty-vt` terminal state
- `tests/xterm-compat` — a generic harness that compares the crate's output against real xterm serialize output

## Dependency chain

- Ghostty C API additions live in `jemdiggity/ghostty`
- Rust wrapper/API changes live in `jemdiggity/libghostty-rs`
- This repo depends on `libghostty-vt` from that fork

## Quick start

```bash
cargo test
node --test tests/xterm-compat/node-runner/reference-runner.test.mjs
node --test tests/xterm-compat/compare/compare.test.mjs
```

To use a local Ghostty checkout instead of the pinned fork commit:

```bash
export GHOSTTY_SOURCE_DIR=/path/to/ghostty
```
