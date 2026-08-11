# AGENTS.md — ecosystem overview for agents working in this repo

This repo (`my-idea`) is one of four in a coordinated ecosystem. If you're
an agent (Codex, Claude Code, or otherwise) picking up work here, read this
first — it saves you from re-deriving context another agent already has.

## The four repositories

- **my-lisp** — the semantic source of truth. Defines the language: parser,
  evaluator, exactness model (rationals, no floats), `lib/core.my` standard
  library. Language contract version 1.0 (`language-contract.my`).
- **fpga-lisp** — hardware implementation of the same language on an FPGA.
  Tracks an ISA contract (`isa-contract.my`, version 0.2) against my-lisp's
  semantics.
- **cml** — an AOT compiler from my-lisp source to fpga-lisp's ISA. Tracks
  conformance against both other repos (`compatibility.my`). Has CI
  (`.github/workflows/`) running real `iverilog` E2E simulation.
- **my-idea** (this repo) — an observer/IDE layer, "System Observatory".
  Depends on my-lisp via a cargo git dependency (`branch = "main"`), not a
  submodule — revision sync is manual only, no CI invariant checks that
  `Cargo.lock`'s pinned my-lisp sha matches what `compatibility.my`/
  `ecosystem-status.my` currently consider current. See
  [docs/system-observatory-vision.md](docs/system-observatory-vision.md)
  for the full vision and
  [docs/system-observatory-status.md](docs/system-observatory-status.md)
  for what's actually implemented today.

## my-idea's role: reader, not source of truth

`my-idea` defines nothing. It reads the other three repos' machine-readable
contracts and shows whether the ecosystem is currently coherent. The
backend aggregator lives in `src-tauri/src/ecosystem/`:

- `git.rs` — finds sibling repos on disk (expects
  `~/Documents/GitHub/{my-idea,my-lisp,fpga-lisp,cml}`, offline-first, no
  GitHub API) and reads their branch/SHA via `git`.
- `contracts.rs` — parses `language-contract.my`, `isa-contract.my`,
  `compatibility.my`, and my-lisp's `ecosystem-status.my` (`cml` entry:
  tier-1 skips, ci-status, equal?/defmacro status) and the canonical
  fixture inventory (`my-lisp/tests/fixtures/conformance.my`) — all via
  my-lisp's own reader (`my_lisp::parse`), not hand-rolled string scanning.
- `mod.rs` — aggregates into `EcosystemStatus`, exposed as the Tauri command
  `ecosystem_status` and surfaced in the desktop UI (ClojureScript,
  `src-cljs/my_idea/core.cljs`) behind the **🔭 Ecosystem** button.

Note: the frontend is **ClojureScript + shadow-cljs**
(`src-cljs/my_idea/`), not Svelte — the original vision doc mentions Svelte
as an earlier plan, but the actual stack diverged.

## Talking to my-lisp live

`my-lisp --tcp[=PORT]` (default 9999) starts a REPL reachable over TCP on
`127.0.0.1` only, per-connection state isolated (fixed 2026-08-11, commit
`c762a0c`). Useful for one-off semantic checks. Opening a raw socket is a
regular but non-trivial action in some agent harnesses (Claude Code's auto
mode has blocked it here before as a safety classifier decision) — if
blocked, prefer running the already-built `my-lisp` CLI binary
(`my-lisp/target/{debug,release}/my-lisp.exe`) directly instead of retrying
the socket path.

## Cross-session coordination protocol (agreed with my-lisp/cml/fpga-lisp)

1. Durable facts go in `ecosystem-status.my`/`ecosystem-status.md` in the
   relevant repo — written after the fact (commit done, CI green), not
   "plan to do X".
2. Direct messages between sessions are for synchronous asks, not restating
   what's already in a status file.
3. Anchor claims to a commit sha or file:line, not a paraphrase from memory.
4. Don't block on confirmation before continuing your own work unless
   there's a real dependency.
5. Verify CI via the REST API (`conclusion` field on
   `api.github.com/repos/.../actions/runs`), never the HTML Actions page —
   cml burned real time on a WebFetch-on-HTML false positive.
6. Manual/static trace is not a substitute for a real run — fpga-lisp's
   M28 was reported manually-verified, then failed on its first real
   iverilog run.

## Shared-checkout hazard

This repo's working directory has been edited concurrently by more than
one agent (this Claude Code session and a separate Codex/OpenCode
orchestration reading `C:\Users\user\Documents\GitHub\docs\`) without a
git worktree boundary — a branch switch by one agent while another has
uncommitted changes is a real risk that already happened once here
(`main` → `agent/test-contract-readers`, 2026-08-11). Before editing:
run `git status -sb` first, and prefer a dedicated `git worktree` over
switching branches in a checkout another agent might be using.
