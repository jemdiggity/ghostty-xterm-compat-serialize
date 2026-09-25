#!/usr/bin/env bash
# Native macOS validation only. No publication or production manifest edits.
set -euo pipefail
ART_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$ART_DIR/../../.." && pwd)"
RUN="$ROOT/.tmp/reproduce-1eca443d"
GHOSTTY_BASE=622b4eecd7d2ce1a10930537c17f0d61abdba817
GHOSTTY_COMMIT=cd43c78ecc559dc82bf65aee332e4911b19a8d03
WRAPPER_BASE=61ae57d7b42821e1b19ad325abc7ff49c80fc880
WRAPPER_COMMIT=f59a104ad6512e2ba8809dd5f37e46c96f01e740
SERIALIZER_BASE=1cde7c0cc70af1962564ab00c22108ab34357aa8
ZIG_SHA=b23d70deaa879b5c2d486ed3316f7eaa53e84acf6fc9cc747de152450d401489
# Refuse to overwrite an earlier reproduction or its evidence.
mkdir -p "$ROOT/.tmp"
mkdir "$RUN"
mkdir "$RUN/logs" "$RUN/toolchain" "$RUN/serializer"
git clone --no-checkout https://github.com/jemdiggity/ghostty.git "$RUN/ghostty"
git -C "$RUN/ghostty" fetch https://github.com/ghostty-org/ghostty.git "$GHOSTTY_BASE"
git -C "$RUN/ghostty" bundle verify "$ART_DIR/ghostty.bundle"
git -C "$RUN/ghostty" fetch "$ART_DIR/ghostty.bundle" refs/heads/task/zig016
git -C "$RUN/ghostty" checkout --detach "$GHOSTTY_COMMIT"
git clone --no-checkout https://github.com/jemdiggity/libghostty-rs.git "$RUN/libghostty-rs"
git -C "$RUN/libghostty-rs" cat-file -e "$WRAPPER_BASE^{commit}"
git -C "$RUN/libghostty-rs" bundle verify "$ART_DIR/libghostty-rs.bundle"
git -C "$RUN/libghostty-rs" fetch "$ART_DIR/libghostty-rs.bundle" refs/heads/task/zig016
git -C "$RUN/libghostty-rs" checkout --detach "$WRAPPER_COMMIT"
# Verify the reviewable patches reproduce exactly the bundled trees.
for repo in ghostty libghostty-rs; do
  if [ "$repo" = ghostty ]; then
    base="$GHOSTTY_BASE"; patch=0001-ghostty-screen-accessors.patch
  else
    base="$WRAPPER_BASE"; patch=0002-libghostty-rs-zig016.patch
  fi
  GIT_INDEX_FILE="$RUN/$repo.index" git -C "$RUN/$repo" read-tree "$base"
  GIT_INDEX_FILE="$RUN/$repo.index" git -C "$RUN/$repo" apply --cached "$ART_DIR/$patch"
  tree="$(GIT_INDEX_FILE="$RUN/$repo.index" git -C "$RUN/$repo" write-tree)"
  test "$tree" = "$(git -C "$RUN/$repo" rev-parse 'HEAD^{tree}')"
done
if [ "${1:-}" = --restore-only ]; then exit 0; fi
curl -fLsS https://ziglang.org/download/index.json -o "$RUN/toolchain/index.json"
python3 - "$RUN/toolchain/index.json" "$ZIG_SHA" <<'PY'
import json, sys
assert json.load(open(sys.argv[1]))['0.16.0']['aarch64-macos']['shasum'] == sys.argv[2]
PY
curl -fLsS https://ziglang.org/download/0.16.0/zig-aarch64-macos-0.16.0.tar.xz -o "$RUN/toolchain/zig.tar.xz"
echo "$ZIG_SHA  $RUN/toolchain/zig.tar.xz" | shasum -a 256 -c -
tar -xf "$RUN/toolchain/zig.tar.xz" -C "$RUN/toolchain"
export PATH="$RUN/toolchain/zig-aarch64-macos-0.16.0:$PATH"
export ZIG_GLOBAL_CACHE_DIR="$RUN/zig-cache"
{ sw_vers; uname -m; xcodebuild -version; xcrun --show-sdk-path; xcrun --show-sdk-version; zig version; rustc --version; cargo --version; node --version; bun --version; } > "$RUN/logs/environment.log"
(cd "$RUN/ghostty" && zig fmt --check src/lib_vt.zig src/terminal/c/main.zig src/terminal/c/terminal.zig && zig build -Demit-lib-vt --prefix "$RUN/out") > "$RUN/logs/native-build.log" 2>&1
(cd "$RUN/ghostty" && zig build test-lib-vt --summary all) > "$RUN/logs/ghostty-tests.log" 2>&1
(cd "$RUN/ghostty" && zig build test-lib-vt -Dtest-filter=screen_ --summary all) > "$RUN/logs/screen-tests.log" 2>&1
# Exercise the normal build.rs fetch path. The commit is unpublished, so
# redirect only this command's Git fetch to the restored exact local commit.
(unset GHOSTTY_SOURCE_DIR; cd "$RUN/libghostty-rs"; \
  GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0="url.file://$RUN/ghostty.insteadOf" \
  GIT_CONFIG_VALUE_0=https://github.com/jemdiggity/ghostty.git \
  CARGO_TARGET_DIR="$RUN/cargo-fetch" \
  cargo test -p libghostty-vt -p libghostty-vt-sys) > "$RUN/logs/wrapper-fetch-tests.log" 2>&1
(cd "$RUN/libghostty-rs" && cargo fmt --all -- --check) > "$RUN/logs/wrapper-format.log" 2>&1
export GHOSTTY_SOURCE_DIR="$RUN/ghostty"
export CARGO_TARGET_DIR="$RUN/cargo-serializer"
git -C "$ROOT" archive "$SERIALIZER_BASE" | tar -x -C "$RUN/serializer"
python3 - "$RUN" <<'PY'
from pathlib import Path
import sys
run = Path(sys.argv[1])
p = run / 'serializer/Cargo.toml'
s = p.read_text() + '\n[patch."https://github.com/jemdiggity/libghostty-rs.git"]\n'
for crate in ['libghostty-vt', 'libghostty-vt-sys']:
    s += f'{crate} = {{ path = "{run / "libghostty-rs/crates" / crate}" }}\n'
p.write_text(s)
PY
(cd "$RUN/serializer" && cargo test --workspace) > "$RUN/logs/serializer-tests.log" 2>&1
(cd "$RUN/serializer/tests/xterm-compat/node-runner" && bun install --frozen-lockfile) > "$RUN/logs/bun.log" 2>&1
(cd "$RUN/serializer" && node --test tests/xterm-compat/node-runner/reference-runner.test.mjs) > "$RUN/logs/reference-tests.log" 2>&1
(cd "$RUN/serializer" && node --test tests/xterm-compat/compare/compare.test.mjs) > "$RUN/logs/comparison-tests.log" 2>&1
echo "Native verification complete. Logs: $RUN/logs"
