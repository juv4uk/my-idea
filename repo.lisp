; repo.my — Swarm Contract v0.1 scope declaration for my-idea.
; See my-lisp/docs/swarm-mesh-v2.md for the full spec. Format confirmed
; by example against fpga-lisp/repo.my and shiva-sutras/repo.my.
;
; A declaration of scope, not an authorization grant -- authorities/
; non-authorities state what this repo is and is not the source of
; truth for, so other repos' agents don't have to re-derive it.
;
; my-idea is the ecosystem's simple WSM/Tauri IDE and build shell. System
; Observatory and swarm control belong to tauricode. my-idea consumes the
; authoritative my-lisp CLI/LSP and fixed Tauri build profiles; it does not
; decide language semantics, compiler behavior or hardware claims.

(repository
  (id my-idea)
  (role wsm-tauri-ide)
  (exports project-editing build-run-interface self-build-evidence)
  (imports language-contract compiler-cli lsp tauri-build-contract)
  (capabilities rust clojurescript tauri gui wasm project-files process-runner)
  (authorities ide-ux workspace-security build-orchestration)
  (non-authorities language-semantics isa-design compiler-internals shiva-canon paninian-ontology hardware-implementation))
