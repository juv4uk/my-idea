#!/usr/bin/env bash
# Builds the sens CLI sidecar for local dev · Збирає sidecar sens для
# локальної розробки · Baut den sens-CLI-Sidecar für die lokale
# Entwicklung.
#
# Tauri's `externalBin` (tauri.conf.json) requires the sidecar binary to
# already exist at `src-tauri/binaries/sens-<host-triple>` before
# `cargo build`/`cargo tauri dev` will even compile — this is a one-time
# (or "whenever external/sens moves") local setup step, not something
# that happens automatically. Release builds use the same
# external/sens gitlink revision in .github/workflows/publish-release.yml;
# this helper simply stages the host-target binary from that same pinned tree
# for local development.
#
# Usage / Використання: bash scripts/build-repl-sidecar.sh

set -euo pipefail

if [[ ! -d external/sens || ! -f external/sens/Cargo.toml ]]; then
  echo "external/sens submodule not checked out — run \`git submodule update --init\` first." >&2
  exit 1
fi

# Точний пін драбини доменів, спільний для GUI, WASM і sidecar.
GITLINK="$(git rev-parse HEAD:external/sens)"
CHECKED_OUT="$(git -C external/sens rev-parse HEAD)"
if [[ "$GITLINK" != "$CHECKED_OUT" ]]; then
  printf 'Помилка піна SENS: gitlink=%s checkout=%s; запустіть git submodule update --init external/sens\n' "$GITLINK" "$CHECKED_OUT" >&2
  exit 1
fi
printf 'PINNED_SENS_GITLINK=%s\n' "$GITLINK"

HOST_TRIPLE="$(rustc -Vv | awk '/^host:/ {print $2}')"
if [[ -z "$HOST_TRIPLE" ]]; then
  echo "could not determine the host target triple from \`rustc -Vv\`." >&2
  exit 1
fi

cargo build --release --manifest-path external/sens/Cargo.toml -p sens-cli --bin sens

mkdir -p src-tauri/binaries
BINARY_NAME="sens"
if [[ "$HOST_TRIPLE" == *windows* ]]; then
  BINARY_NAME="sens.exe"
fi
DEST="src-tauri/binaries/sens-${HOST_TRIPLE}"
if [[ "$HOST_TRIPLE" == *windows* ]]; then
  DEST="${DEST}.exe"
fi
cp "external/sens/target/release/${BINARY_NAME}" "$DEST"
echo "Sidecar ready at $DEST"
