# my-idea implementation plan

This plan executes [ADR-003](ADR-003-SIMPLE-SELF-BUILDING-IDE.md).

## Reuse map

| Source | Reuse | Boundary |
|---|---|---|
| existing `my-idea` | CodeMirror, tabs, dirty state, project tree, layout, Tauri v2 shell, i18n | keep and simplify |
| `my-lisp` | CLI execution, LSP diagnostics/completion, language semantics | consume; never duplicate |
| `cml` | future stable compiler CLI | blocked until a CLI contract exists |
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
- Add a CML Compile action only after `CML-STABLE-CLI-CONTRACT` is ratified.

The CML contract must define, before IDE integration:

- exact executable and argv syntax;
- named target selection and output artifact path;
- structured diagnostics and exit-status meanings;
- typed unsupported reasons rather than a generic green skip;
- the consumed target-ABI schema/version/digest for freestanding output.

Current CML backend-classification debt must be closed independently; an IDE
adapter must not turn `panic`, generic `Unsupported`, or documentation drift
into a successful Compile result.

### P3 — Tauri development loop

- Detect `package.json`, `src-tauri/Cargo.toml` and Tauri configuration.
- Expose Check, Dev and Build through fixed Bun/Tauri command profiles.
- Show missing-tool errors as one actionable diagnostic.
- Preserve live stdout, stderr, exit status and cancellation evidence.

### P4 — self-build witness

- Open `my-idea` itself.
- Build it through the same public adapter used for other Tauri projects.
- Produce and identify a real bundle.
- Record commit, toolchain, command profile, artifact path and SHA-256 digest.

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
