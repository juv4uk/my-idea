# CLAUDE.md — context for AI assistants working on my-lisp

This file travels with the code (unlike personal/session memory, which stays local to one machine). Read it first when starting work on this branch or on the future standalone my-lisp repo it will become.

## What my-lisp is

**"A small language that grows itself."** A Lisp built around McCarthy's seven primitives (`quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond`), plus the minimal semantic kernel needed to bootstrap everything else: `lambda`, `def`, `defmacro`. Everything derivable from that kernel — `identity`, `not`, `pair`, `second`, `third`, `caar`, `cadr` — is written in my-lisp itself (`lib/core.my`), not added as Rust built-ins. Full rationale: `docs/language-core.md`.

## Non-negotiable design principles

- **Bootstrap boundary**: Rust (or C, or HDL — see below) provides only what it does exceptionally well — safe values, parsing, lexical closures, deterministic evaluation, stack control, diagnostics, explicit capability boundaries. Higher-level forms belong in the language itself. Don't grow the host-language built-in surface when the existing kernel can already express something.
- **Exact rational arithmetic is a core purpose, not a nice-to-have.** `/` on integers/rationals must stay exact (`5/336`, not `0.0148...`) — see Racket's exact/inexact distinction in the codebase's own conventions. One inexact operand makes a result inexact; never silently approximate an exact conversion. This holds across every implementation of the language, including future non-Rust ones.
- **`.my` is the canonical source extension**; `.lisp` is a compatible alias. Don't introduce `.myl`.
- **Trilingual documentation and commit messages** (English / Ukrainian / German) is this project's convention — see any existing doc or commit in the git log for the pattern.

## Current state (as of this branch's creation, 2026-08-08)

This branch/repo currently mirrors `my-idea`'s `crates/my-lisp*` workspace members:
- `crates/my-lisp` — the canonical Rust core. `eval/` is split into `mod.rs` (trampoline + dispatch), `arithmetic.rs`, `special_forms.rs`, `closures.rs`. 40 tests.
- `crates/my-lisp-cli` — the `my-lisp` binary (REPL + file runner). 8 tests.
- `crates/my-lisp-wasm` — WASM bindings powering both the web IDE's Language Lab and the standalone `my-lisp-cli-web.html` REPL.
- `crates/my-lisp-literate` — literate-Markdown source-offset mapping. 4 tests.
- `lib/core.my` — the bootstrapped standard library.
- `docs/quote-tutorial.md` — a beginner walkthrough of homoiconicity (write code with a leading `'`, delete it, watch data become a running program).
- `docs/testing.md` — current test counts and what each suite covers.

## Confirmed future direction (not yet started — ask before scaffolding)

`my-lisp` is meant to eventually branch out of `my-idea` into **two separate repositories**, parallel and independent, neither a "lesser" fallback of the other:

1. **Rust + Lisp** — this line of work, continuing as the reference implementation.
2. **C + Lisp** — targets embedded/microcontroller environments where the Rust runtime or WASM doesn't fit. Must support real bignum-capable exact rational arithmetic (not fixed-point/float fallback) — this is a genuinely hard embedded-C problem and should be scoped explicitly, not assumed away.

A third, related but distinct thread: a **custom Lisp-oriented HDL core** (bespoke gateware, not a RISC-V or generic soft-core CPU) targeting the **Sipeed Tang Primer 25K** FPGA board — a from-scratch "Lisp machine," not a C program running on a soft CPU. The arithmetic is planned as **multi-cycle (4 clock cycles per operation)**, not single-cycle combinational logic — this is a settled design decision, not an open question. How this HDL work relates to the C+Lisp repo (same thing? separate strand?) is **not yet confirmed** — ask rather than assume.

Watch for the McCarthy-primitive contract and the exact-rational-arithmetic principle needing to hold across every implementation — conformance tests (`crates/my-lisp/tests/mccarthy.rs`'s `conformance_tests_from_json`, backed by `tests/fixtures/conformance.json`-style fixtures) are the natural shared, implementation-independent contract to extend.

**Don't start scaffolding a new repo, the C core, or the HDL work unprompted.** This file exists so the *why* and the *already-decided* parts survive a repo split — not as a green light to start building them.
