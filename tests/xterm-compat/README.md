# Ghostty xterm Serialize Compatibility Experiment

Reference runner:

```bash
node tests/xterm-compat/node-runner/reference-runner.mjs startup_prompt
```

Ghostty runner:

```bash
cd tests/xterm-compat/ghostty-runner && cargo run -- startup_prompt
```

Comparison tests:

```bash
node --test tests/xterm-compat/node-runner/reference-runner.test.mjs
node --test tests/xterm-compat/compare/compare.test.mjs
cd tests/xterm-compat/ghostty-runner && cargo test
```

Success levels:

1. Exact serialize compatibility
2. Semantic restore compatibility

Known boundary:

- `left_right_margin_active` is intentionally not an exact match. xterm/headless does not model DEC left/right margin mode (`?69h` / `DECSLRM`) the way Ghostty does, so this fixture is kept as an expected incompatibility marker.
