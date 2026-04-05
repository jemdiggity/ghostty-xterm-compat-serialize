# xterm Compatibility Fixtures

Deterministic VT input fixtures shared by the xterm reference runner and the Ghostty compatibility runner.

Each file is a JSON object with:

- `name`
- `description`
- `chunks`: ordered `{ delayMs, data }` entries
