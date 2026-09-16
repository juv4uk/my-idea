# Compiling with cml directly in my-idea

(Secondary/English mirror — see `cml-compiler-2026-09-16.uk.md` for the
primary version.)

my-idea can compile a `.lisp` file through the real **cml** compiler
(`juv4uk/cml`) into a native x86-64 freestanding ELF and run the result —
the **"🔧 Compile (cml)"** button next to "⚡ Evaluate" and "▶ Run".

Like the plugin bridge (#41), `CompilerBridge`/`CompilerBuildAdapter`
(issue #16) already existed, fully contract-tested, but unreachable from
any Tauri command or UI button. This wires it up for the first time.

## Three distinct actions

- **⚡ Evaluate** — in-process tree-walking interpreter.
- **▶ Run** — the real `my-lisp` CLI interpreter as an external process.
- **🔧 Compile (cml)** — compiles through cml, our separate, authoritative
  native compiler, and runs the resulting artifact.

my-idea never reimplements cml's semantics — it only runs `cml` as an
external process and surfaces its diagnostics verbatim.

## Honest boundary

cml today is a **freestanding x86-64 backend, pure computation only**.
`(+ 40 2)` compiles and runs fine. Side effects like `print` aren't
supported by this backend yet (no libc/stdio in freestanding mode) — it
fails with `unsupported IR in x86_64-freestanding backend: Var (unbound)`.

Verified against a real ELF binary
(`src-tauri/tests/real_cml_compiler_contract.rs`), but currently useful
for a narrow class of programs. Growing cml's capabilities is cml's own
work, not my-idea's.

## Binary resolution

Same pattern as the `my-lisp` sidecar: `MY_IDEA_CML_BIN` env override →
sibling `../cml/target/release/cml` → `cml` on PATH.
