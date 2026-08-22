;; Guix manifest for my-idea's dev environment.
;; Reproducible toolchain for the Tauri (Rust) + shadow-cljs (ClojureScript)
;; + wasm-pack stack this repo builds with — see package.json / Cargo.toml.
;;
;; Usage:
;;   guix shell -m manifest.scm
;; then, since nss-certs doesn't wire itself in automatically:
;;   export SSL_CERT_DIR="$GUIX_ENVIRONMENT/etc/ssl/certs"
;; and, before any cargo build/check/test (DrvFs fchmod EPERM otherwise —
;; see AGENTS.md's "Known fix: cargo check/cargo build need CARGO_TARGET_DIR
;; off DrvFs"):
;;   export CARGO_TARGET_DIR="$HOME/.cache/my-idea-target"
;;
;; rustc-too-old blocker: see AGENTS.md's "If `cargo check` fails with
;; 'rustc X is not supported'" — resolved via `guix time-machine` against
;; ../my-lisp/channels.scm, not by changing this manifest.
;;
;; Bun (the JS package manager/runner, see AGENTS.md) is NOT in this list —
;; `guix search bun` finds nothing in this channel as of 2026-08-12, so it
;; isn't packaged here yet. Install it via the official installer instead
;; (https://bun.sh/install) and put ~/.bun/bin on $PATH; `node`/`npm` stay
;; in this manifest since `bun run build`/`bun run test` still shell out to
;; `node` internally (see AGENTS.md's Bun section). Packaging Bun for Guix
;; is a real follow-up, not done as part of this migration.
;;
;; Same class of gap, confirmed 2026-08-22: "wasm-pack" is not packaged in
;; this channel either, and Guix's rust has NO wasm32-unknown-unknown cross
;; target, so `npm run build`/`npm run wasm` cannot run inside this profile
;; alone. Working solution (see AGENTS.md's wasm-toolchain section):
;; machine-local rustup (~/.cargo) + wasm-pack binary (~/.local/bin), with
;; both dirs prepended to PATH inside the guix shell. Not solvable by
;; editing this manifest today.

(specifications->manifest
 '("rust"
   "rust:cargo"
   "nss-certs"        ; TLS root certs — cargo/npm need these to hit crates.io/registry.npmjs.org
   "node"
   "openjdk"          ; shadow-cljs runs on the JVM
   "git"
   "pkg-config"
   ;; Tauri v2's Linux runtime deps. Plain "webkitgtk" resolves to the
   ;; GTK4/WebKit6 build (javascriptcoregtk-6.0.pc) — Tauri's
   ;; javascriptcore-rs-sys wants the GTK3 line (javascriptcoregtk-4.1.pc),
   ;; provided by "webkitgtk-for-gtk3" specifically.
   "webkitgtk-for-gtk3"
   "gtk+"
   "libappindicator"))

;; AppImage bundling (tauri build's third Linux bundle format, after
;; .deb/.rpm) shells out to xdg-open at bundle time — without it that one
;; step fails even though .deb/.rpm already succeeded. "xdg-utils" IS
;; packaged in Guix, but adding it to this manifest broke pkg-config
;; resolution for gio-sys (profile rebuild left `pkg-config` unfindable
;; on PATH — not yet root-caused, likely a propagated-input ordering
;; issue with xdg-utils' own dependencies). Install it natively instead:
;;   sudo apt-get install -y xdg-utils
;; same pattern as Bun and build-essential/rustup elsewhere in this repo.
