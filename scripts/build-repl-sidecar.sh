#!/usr/bin/env bash
# Builds the my-lisp CLI sidecar for local dev · Збирає sidecar my-lisp для
# локальної розробки · Baut den my-lisp-CLI-Sidecar für die lokale
# Entwicklung.
#
# Tauri's `externalBin` (tauri.conf.json) requires the sidecar binary to
# already exist at `src-tauri/binaries/my-lisp-<host-triple>` before
# `cargo build`/`cargo tauri dev` will even compile — this is a one-time
# (or "whenever external/my-lisp moves") local setup step, not something
# that happens automatically. The actual release build instead fetches
# my-lisp's latest main fresh in CI (.github/workflows/publish-release.yml)
# — this script uses the local external/my-lisp submodule checkout instead,
# since that's what a dev checkout already has on disk.
#
# Usage / Використання: bash scripts/build-repl-sidecar.sh

set -euo pipefail

if [[ ! -d external/my-lisp || ! -f external/my-lisp/Cargo.toml ]]; then
  echo "external/my-lisp submodule not checked out — run \`git submodule update --init\` first." >&2
  exit 1
fi

HOST_TRIPLE="$(rustc -Vv | awk '/^host:/ {print $2}')"
if [[ -z "$HOST_TRIPLE" ]]; then
  echo "could not determine the host target triple from \`rustc -Vv\`." >&2
  exit 1
fi

cargo build --release --manifest-path external/my-lisp/Cargo.toml -p my-lisp-cli --bin my-lisp

mkdir -p src-tauri/binaries
BINARY_NAME="my-lisp"
if [[ "$HOST_TRIPLE" == *windows* ]]; then
  BINARY_NAME="my-lisp.exe"
fi
DEST="src-tauri/binaries/my-lisp-${HOST_TRIPLE}"
if [[ "$HOST_TRIPLE" == *windows* ]]; then
  DEST="${DEST}.exe"
fi
cp "external/my-lisp/target/release/${BINARY_NAME}" "$DEST"
echo "Sidecar ready at $DEST"
