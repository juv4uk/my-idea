;; Guix manifest for my-idea's dev environment.
;; Reproducible toolchain for the Tauri (Rust) + shadow-cljs (ClojureScript)
;; + wasm-pack stack this repo builds with — see package.json / Cargo.toml.
;;
;; Usage:
;;   guix shell -m manifest.scm
;; then, since nss-certs doesn't wire itself in automatically:
;;   export SSL_CERT_DIR="$GUIX_ENVIRONMENT/etc/ssl/certs"
;;
;; Known blocker (2026-08-12): Guix's `rust` is 1.85.1; several transitive
;; deps of the Tauri stack (darling, icu_*, time, zbus, plist) need 1.86-1.88.
;; `cargo check` fails until either Guix ships a newer rust or Cargo.lock
;; pins those crates back to 1.85.1-compatible versions.

(specifications->manifest
 '("rust"
   "rust:cargo"
   "nss-certs"        ; TLS root certs — cargo/npm need these to hit crates.io/registry.npmjs.org
   "node"
   "openjdk"          ; shadow-cljs runs on the JVM
   "git"
   "pkg-config"
   ;; Tauri's Linux runtime deps (webkitgtk, gtk, etc.)
   "webkitgtk"
   "gtk+"
   "libappindicator"))
