;; Guix manifest for my-idea's dev environment.
;; Reproducible toolchain for the Tauri (Rust) + shadow-cljs (ClojureScript)
;; + wasm-pack stack this repo builds with — see package.json / Cargo.toml.
;;
;; Usage: guix shell -m manifest.scm

(specifications->manifest
 '("rust"
   "rust:cargo"
   "node"
   "openjdk"          ; shadow-cljs runs on the JVM
   "git"
   "pkg-config"
   ;; Tauri's Linux runtime deps (webkitgtk, gtk, etc.)
   "webkitgtk"
   "gtk+"
   "libappindicator"))
