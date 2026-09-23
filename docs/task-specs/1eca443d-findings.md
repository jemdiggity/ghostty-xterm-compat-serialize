# Zig 0.16 paired dependency upgrade — task 1eca443d

Verified on the Mac Studio, 2026-09-23. The current Ghostty + wrapper pair
builds with official Zig 0.16.0 against Xcode 27 / macOS SDK 27.0. Serializer
source changes are unnecessary; its production manifests and lockfiles remain
unchanged pending publication. **This is not a Kanna Bazel/release unblock.**

## Publication gate

These exact local commits are ready for review and publication, in this order:

| Fork | Commit to publish | Parent |
| --- | --- | --- |
| `jemdiggity/ghostty` | `cd43c78ecc559dc82bf65aee332e4911b19a8d03` | upstream `622b4eecd7d2ce1a10930537c17f0d61abdba817` |
| `jemdiggity/libghostty-rs` | `f59a104ad6512e2ba8809dd5f37e46c96f01e740` | `61ae57d7b42821e1b19ad325abc7ff49c80fc880` |

The wrapper's `GHOSTTY_COMMIT` already names the first commit and retains the
public fork URL. Neither commit has been pushed. Publish on new branches
(e.g. `jemdiggity/zig016-screen-api` and `jemdiggity/zig016-xterm-compat`) to
avoid rewriting existing fork branches. The wrapper's default network fetch
cannot work until the Ghostty commit is reachable from its fork.

After both exact refs exist and publication is authorized, update the serializer's
`libghostty-vt` git rev in `crates/ghostty-xterm-compat-serialize/Cargo.toml`
to the wrapper commit, regenerate the workspace lockfile (and reconcile the
tracked legacy runner lockfile), and rerun the tests without local overrides.
The runner manifest depends on the serializer by path and needs no git rev.
If publication rewrites either SHA, repin the wrapper first, then use the newly
published wrapper SHA for the serializer. Publish the resulting serializer
commit before moving Kanna's serializer pin.

At that point create or identify a dependent Kanna task to update
`crates/daemon/Cargo.toml` and `MODULE.bazel` / `rules_zig` together, including
Zig 0.16.0 and its official archive checksum. That task must validate the real
hermetic Bazel build on the affected Mac Studio/MBP SDK path, preserve any
existing owner on-device approval requirements, and record exact SDK, Zig,
Ghostty, wrapper and serializer refs. No Kanna worktree was changed here.

A published wrapper branch `kanna-vendor-ghostty-zig-deps-61ae57d` exists at
`03c9702f6515305b0d19669596f904691df8c341` (two commits beyond the wrapper base).
It contains Ghostty dependency overlays and a runtime manifest-directory fix.
Those old-version overlays are not part of this Cargo-tested pair: the Kanna
Bazel follow-up must adapt its hermetic vendoring for the new Ghostty dependency
graph, rather than copying the old `build.zig.zon` over the new source.

## Recovery and resolved refs

Read first from the surviving local branch at
`229de164e44b0b639086c8db88463017770e50e9`:
`docs/task-specs/362c3351-findings.md`, patches 0001/0002/0003, and `reproduce.sh`.
Imported source content with `git apply`, not the old task commits or their
trailers. New commits have no attribution/session trailers.

Before edits: `git fetch origin main`, then `git rev-parse HEAD origin/main`
both returned `1cde7c0cc70af1962564ab00c22108ab34357aa8`. Working tree was clean.
Resolved current remotes using `git ls-remote` and task-local clones:

| Repository / branch | SHA |
| --- | --- |
| serializer `origin/main` | `1cde7c0cc70af1962564ab00c22108ab34357aa8` |
| upstream Ghostty `main` | `622b4eecd7d2ce1a10930537c17f0d61abdba817` |
| fork Ghostty `main` | `ba398dfff3e30ff83da07140981ca138410cf608` |
| fork Ghostty `jemdiggity/libghostty-screen-api` | `665a03f380204ce1976941d36649963b4da80880` |
| wrapper `master` | `811cbdd85a99b40a63424a9d247c4f3653c11924` |
| wrapper `jemdiggity/xterm-compat-serializer` | `61ae57d7b42821e1b19ad325abc7ff49c80fc880` |

Historical evidence reused, not rerun or claimed as new: the September 6 spike
used upstream `492300cad104195411d12217dd22f1cd05f31376`, Xcode 26.6 / SDK 26.5,
and Zig 0.16.0. Its native build, wrapper tests, 46 comparisons and byte-identical
JSON on all 44 baseline fixtures passed. The supplied old-stack SDK 27 failure
(`INFINITY` in Zig 0.15.2's libc++) was not rerun. This task tests the changed
upstream revision and SDK; it does not claim a fresh old/new JSON identity check.

## Companion changes

Ghostty retains `ghostty_terminal_screen_get` and
`ghostty_terminal_screen_grid_ref`, their exports and four recovered tests.
The recovered patch applies cleanly to the current upstream. One additional
adaptation handles upstream's `cursor_at_prompt`: the explicit-screen getter
now inspects the requested screen instead of the active one. A new test checks
an inactive primary prompt, alternate screen, and normal active-screen getter.

The wrapper retains the recovered signed-enum/API adaptations, selection-null
formatter option and removal of redundant `.zig-cache` archive scraping.
Bindings were freshly regenerated from current patched headers using the
repository's `gen-bindings` tool and formatted; README/agent toolchain
requirements now say 0.16.0. The stale spike pin/comment is replaced with the
actual paired Ghostty commit. No unrelated wrapper APIs were added.

## Environment and commands

- arm64 Mac Studio, macOS 27.0 (`26A428`).
- Xcode 27.0 (`27A266a`), selected SDK 27.0 via `xcrun --show-sdk-version`.
- `SDKROOT` and `DEVELOPER_DIR` unset; system selection unchanged.
- Build-generated `libc.txt` confirms
  `sys_include_dir=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX27.0.sdk/usr/include`.
- Homebrew Zig remains 0.15.2. Downloaded official
  `https://ziglang.org/download/0.16.0/zig-aarch64-macos-0.16.0.tar.xz`
  into `.tmp/toolchain/`; verified SHA-256 against freshly fetched official
  `https://ziglang.org/download/index.json`:
  `b23d70deaa879b5c2d486ed3316f7eaa53e84acf6fc9cc747de152450d401489`.
- rustc/cargo 1.93.1, Node v24.15.0, Bun 1.3.10.

Commands below ran from this worktree, with native source/build operations
inside `.tmp/repos/ghostty` and `.tmp/repos/libghostty-rs`:

```sh
export PATH="$PWD/.tmp/toolchain/zig-aarch64-macos-0.16.0:$PATH"
export ZIG_GLOBAL_CACHE_DIR="$PWD/.tmp/zig-global-cache"
export GHOSTTY_SOURCE_DIR="$PWD/.tmp/repos/ghostty"
# Ghostty cwd:
zig fmt --check src/lib_vt.zig src/terminal/c/main.zig src/terminal/c/terminal.zig
zig build -Demit-lib-vt --prefix ../../out/lib-vt
zig build test-lib-vt --summary all
zig build test-lib-vt -Dtest-filter=screen_ --summary all
# Wrapper cwd, CARGO_TARGET_DIR=<root>/.tmp/cargo-wrapper:
GHOSTTY_INCLUDE_DIR=<root>/.tmp/repos/ghostty/include \
  cargo run -p libghostty-vt-sys --features bindgen-tool --bin gen-bindings
cargo fmt --all
cargo fmt --all -- --check
cargo test -p libghostty-vt -p libghostty-vt-sys
```

Also tested a fresh wrapper build in `.tmp/cargo-fetch` with
`GHOSTTY_SOURCE_DIR` unset. Only that command received:

```sh
GIT_CONFIG_COUNT=1
GIT_CONFIG_KEY_0=url.file://<root>/.tmp/repos/ghostty.insteadOf
GIT_CONFIG_VALUE_0=https://github.com/jemdiggity/ghostty.git
```

This exercises `build.rs` clone/checkout/stamp/build/link using the actual
`GHOSTTY_COMMIT`; fetched `ghostty-src` HEAD was verified as `cd43c78e...`.
It is not a test of remote availability of the unpublished commit.

Serializer was a `git archive HEAD` copy under `.tmp/serializer`, with a local
Cargo `[patch]` pointing at the wrapper. The real manifests/lockfiles were never
rewritten. With `CARGO_TARGET_DIR=<root>/.tmp/cargo-serializer`, ran:

```sh
cargo test --workspace
(cd tests/xterm-compat/node-runner && bun install --frozen-lockfile)
node --test tests/xterm-compat/node-runner/reference-runner.test.mjs
node --test tests/xterm-compat/compare/compare.test.mjs
```

## Results

| Check | Result |
| --- | --- |
| Native static library and dylib | Exit 0 on SDK 27; `otool -L` shows `libSystem.B.dylib`; `nm -gU` finds both screen accessors |
| Full Ghostty lib-vt suite | Exit 0, 6,429 passed / 52 skipped (2,993 + 3,436 passed in two binaries) |
| Final screen-focused suite | Exit 0, 74/74 (33 + 41); includes new prompt test |
| Wrapper tests, source override | Exit 0, 3 + 2 unit tests, 11 doctests, 1 ignored |
| Wrapper tests, default fetch path with local URL redirect | Same passing counts, exit 0 |
| Serializer workspace tests | Exit 0, 4 tests |
| Reference runner | Exit 0, 1/1 |
| xterm comparison harness | Exit 0, 46/46, including the expected left/right-margin incompatibility assertion |
| Zig/Rust formatting and companion diff whitespace | Clean |
| Patch/bundle restoration | Both exact SHAs and trees restored successfully; script syntax valid |

The full suite started on the initially recovered port. The final prompt-query
adaptation was then verified by rebuilding the native library and the focused
screen suite; unrelated full-suite evidence remains applicable. Linux, Windows,
cross-target behavior, MBP and hermetic Bazel were not validated here.

## Durable artifacts

[1eca443d-artifacts](1eca443d-artifacts/) contains two reviewable format-patches,
two incremental Git bundles preserving exact commit IDs, a reproduction script,
and verification logs. Bundles require the parent commits listed above; the
script fetches them from the authoritative repositories. This avoids losing the
paired refs when `.tmp/` is cleaned, or changing their IDs with `git am`.

`reproduce.sh --restore-only` was executed and verified both patches against the
bundled trees. Full reproduction commands were executed as recorded above; the
combined script's full build mode has not been separately rerun. It creates a
fresh `.tmp/reproduce-1eca443d` directory and refuses to overwrite an existing
run. Remove or move that task-local reproduction directory before a second run.
Nothing in the script pushes commits or edits the publishable serializer manifest.
