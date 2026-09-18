# my-idea implementation plan

This plan executes [ADR-003](ADR-003-SIMPLE-SELF-BUILDING-IDE.md).

## Reuse map

| Source | Reuse | Boundary |
|---|---|---|
| existing `my-idea` | CodeMirror, tabs, dirty state, project tree, layout, Tauri v2 shell, i18n | keep and simplify |
| `my-lisp` | CLI execution, LSP diagnostics/completion, language semantics | consume; never duplicate |
| `cml` | pinned `cml-compile` host boundary | consume exact compiler provenance; current executable target is `x86_64-linux` |
| `tauricode` | path-normalization, file-tree and process-cancellation design ideas | adapt narrowly; no Observatory/UI transplant |
| Bun/Cargo/Tauri | frontend checks and native application build | fixed command profiles |

Tauricode is MIT-licensed but carries its own copyright notice. Substantial
copied code requires preservation of that notice. Prefer small independent
implementations of the proven patterns over importing its large OpenCode-
specific dependency graph.

## Execution order

### P0 — truthful product surface

- Keep Open, Save and Save As.
- Present Build, Run and Stop only when their adapter exists and is available.
- Remove Observatory, Oracle, Compare, Swarm and Knowledge Graph controls from
  the primary toolbar.
- Replace the generic AST/right pane with one bottom Build Output pane; retain
  WSM-specific parsed views only as an explicitly WSM view.
- Fix New File -> Save so a new file can be created safely inside the selected
  workspace.

### P1 — project and process substrate

- Detect WSM and Tauri projects from explicit files, not directory-name
  guesses.
- Add a Rust process service with one active build per workspace.
- Pass executable and arguments separately; never concatenate project paths
  into a shell command.
- Stream versioned events containing run ID, stream, line and sequence/time.
- Support bounded cancellation. On Windows terminate the process tree, not
  only the parent shell.

### P2 — WSM development loop

- Use `my-lisp` CLI for the commands it actually supports.
- Use WsmLS/my-lisp LSP for diagnostics, completion and symbols.
- Map diagnostics to files and locations without re-parsing WSM in the IDE.
- The CML Compile action uses the ratified `cml-compile x86-elf <source> <artifact>` boundary; `my-idea` remains a mechanism-only client.

### P2A — shared language-server client

- Implement one persistent, stdio-framed JSON-RPC/LSP client in the Tauri
  backend; do not parse WSM or Rust in the IDE.
- Route `.wsm`, `.my` and `.lisp` to `my-lisp lsp` (WsmLS).
- Route `.rs` to the installed `rust-analyzer` binary.
- Keep one server session per workspace and language, send incremental document
  lifecycle notifications, and stop sessions when the workspace closes.
- Feed diagnostics and completion into CodeMirror through one versioned IDE
  schema. Preserve each server's diagnostic source and message.
- Missing server binaries are actionable diagnostics, never silent fallback to
  the old hard-coded completion list.

Current CML integration status:

- executable/argv is fixed as `cml-compile x86-elf <source> <artifact>`;
- the production bridge records exact compiler/input/artifact provenance and fails closed when the compiler is unavailable;
- `x86_64-linux` is the currently proven IDE target;
- source-location diagnostic transport remains owned upstream by `cml#132` / PR #133; the IDE must not infer locations by parsing prose;
- backend unsupported/error classification remains upstream CML work and must never be converted into a successful Compile result by the IDE.

### P3 — Tauri development loop

- Detect `package.json`, `src-tauri/Cargo.toml` and Tauri configuration.
- Expose Check, Dev and Build through fixed Bun/Tauri command profiles.
- Show missing-tool errors as one actionable diagnostic.
- Preserve live stdout, stderr, exit status and cancellation evidence.

### P4 — self-build spine

- **#12 plan:** emit a deterministic `self-build-plan-v1` before execution. It records clean source HEAD, the single pinned my-lisp SHA, exact `cml-compile` revision, Rust/Bun/Tauri tool versions, fixed `x86_64-linux` compiler target, ordered CML → frontend → Tauri stages, and a SHA-256 plan digest.
- **#13 execution:** make the compiler artifact a fresh required input to the Tauri build/bundle stage; stale or missing compiler output must fail closed.
- **#14 generation witness:** generation 0 builds generation 1; generation 1 itself initiates generation 2; record exact provenance for both transitions.
- Keep the claim at **self-building using our compiler + Tauri** until stronger bootstrap independence is physically demonstrated.

### Later — wsm-os target profile

After the desktop loop and CML CLI contract are proven, `wsm-os` may appear as
an explicit WSM build target. The IDE will consume its evidence states and
artifacts; it will not absorb boot/runtime ownership. QEMU parity is distinct
from physical-hardware parity, and both must remain visibly distinct in build
results.

## Deferred

- Observatory and swarm control: Tauricode.
- Full interactive terminal/PTY.
- Visual form designer.
- Plugin marketplace.
- Multi-project build graph.
- Mobile self-build.
- Arbitrary user-defined shell commands.

## Evidence ladder

```text
PRODUCT-BOUNDARY-RECORDED
-> TOOLBAR-TRUTH-PASS
-> NEW-FILE-SAVE-PASS
-> PROCESS-LIFECYCLE-PASS
-> WSM-RUN-PASS
-> TAURI-BUILD-PASS
-> SELF-BUILD-ARTIFACT-PASS
```

No state proves the next one.
