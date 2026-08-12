;; Guix manifest for my-idea's dev environment.
;; Reproducible toolchain for the Tauri (Rust) + shadow-cljs (ClojureScript)
;; + wasm-pack stack this repo builds with — see package.json / Cargo.toml.
;;
;; Usage:
;;   guix shell -m manifest.scm
;; then, since nss-certs doesn't wire itself in automatically:
;;   export SSL_CERT_DIR="$GUIX_ENVIRONMENT/etc/ssl/certs"
;;
;; rustc-too-old blocker: see AGENTS.md's "If `cargo check` fails with
;; 'rustc X is not supported'" — resolved via `guix time-machine` against
;; ../my-lisp/channels.scm, not by changing this manifest.

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
