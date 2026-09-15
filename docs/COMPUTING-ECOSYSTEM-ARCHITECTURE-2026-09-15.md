# Computing Ecosystem — Architecture, Dependencies and Flow (2026-09-15)

![My Computing Ecosystem — Architecture, Dependencies and Flow](architecture/computing-ecosystem-architecture.png)

Owner-authored architecture diagram of the full stack, from Lisp semantics
down to hardware. Per ADR-003, `my-idea` is a simple WSM/Tauri IDE — the
current programmable desktop body of the Lisp Machine (Open → Edit →
Build/Run/Stop → Output), providing host mechanism (Tauri/Rust desktop
shell, CodeMirror/CLJS editor UI, filesystem/process/network integration,
compiler/REPL transport) around `my-lisp` and `CML`. It does not own
language meaning or invent separate capability semantics.

Full stack, top to bottom: User/Developer → **my-lisp** (Language &
Semantics, owns Canon/meaning) → **CML** (Compiler for My Lisp) → Target
Runtime/OS (e.g. `wsm-os-lisp`, implements the ABI) → Hardware. `my-idea`
sits alongside this stack as a host/tooling layer, not as a compiler or
runtime stage.

This diagram is a snapshot of intent, not itself an authority document —
`juv4uk/ecosystem#7` (the living cross-repo map) remains the authoritative,
updated-in-place source of truth about current ownership and boundaries.
