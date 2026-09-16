#!/usr/bin/env bash
# One-time machine-local dev tool bootstrap · Одноразове встановлення
# інструментів розробки на машину · Einmaliges lokales Setup der
# Entwicklungswerkzeuge.
#
# This installs the two tools Guix's manifest.scm deliberately does NOT
# cover (AGENTS.md's "JavaScript package manager: Bun" and "Known fix:
# npm run build needs rustup + wasm-pack outside the Guix profile"
# sections): Bun itself, and a rustup Rust toolchain with the
# wasm32-unknown-unknown target plus wasm-pack. Neither belongs in
# manifest.scm — Guix doesn't package Bun at all, and Guix's own Rust ships
# without cross-compilation targets — so this script lives outside the
# reproducible environment on purpose, as documented machine setup, not a
# substitute for `guix shell -m manifest.scm`.
#
# Idempotent: safe to re-run; skips anything already installed.
#
# Usage / Використання: bash scripts/bootstrap-dev-environment.sh

set -euo pipefail

echo "== my-idea dev environment bootstrap =="

if [[ -x "$HOME/.bun/bin/bun" ]]; then
  echo "Bun already installed at ~/.bun/bin/bun — skipping."
else
  echo "Installing Bun..."
  curl -fsSL https://bun.sh/install | bash
fi

if [[ -x "$HOME/.cargo/bin/rustup" ]]; then
  echo "rustup already installed at ~/.cargo/bin/rustup — skipping install."
else
  echo "Installing rustup (minimal profile, stable toolchain)..."
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
fi

echo "Ensuring wasm32-unknown-unknown target is installed..."
"$HOME/.cargo/bin/rustup" target add wasm32-unknown-unknown

if [[ -x "$HOME/.local/bin/wasm-pack" ]]; then
  echo "wasm-pack already installed at ~/.local/bin/wasm-pack — skipping."
else
  echo "Installing wasm-pack..."
  mkdir -p "$HOME/.local/bin"
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

cat <<'EOF'

== Done ==

Add these to PATH (in ~/.bashrc / ~/.profile, if not already there):

  export PATH="$HOME/.bun/bin:$HOME/.cargo/bin:$HOME/.local/bin:$PATH"

Everything else (shadow-cljs, cargo/rustc for src-tauri and the my-lisp
crates, Tauri's Linux system libraries) still comes from
`guix shell -m manifest.scm` — this script only covers what Guix
deliberately doesn't package. See AGENTS.md for the full environment model.
EOF
