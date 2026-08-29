# ADR-003: Simple self-building IDE

**Status:** ACCEPTED  
**Date:** 2026-08-30  
**Authority:** direct owner decision

## Decision

`my-idea` is a small, practical IDE for WSM and Tauri projects. Its primary
loop is deliberately Lazarus-like in simplicity, without copying Lazarus's
visual design:

```text
Open project -> Edit -> Build or Run -> Stop -> Read output
```

The product surface is:

```text
Project tree | Editor
-------------+-------
Build output / diagnostics
```

Only end-to-end working operations may appear as primary toolbar actions.

The System Observatory, swarm dashboard, ecosystem knowledge graph and agent
control-plane features belong to `tauricode`. Their existing implementation in
`my-idea` is historical and may remain temporarily behind the product surface
while removal is verified, but it is no longer the direction of this repo.

## Product boundary

`my-idea` owns:

- local project and file editing;
- the CodeMirror editor experience;
- explicit Build, Run, Stop and Clean operations;
- a bounded build-output and diagnostics panel;
- WSM integration through the authoritative `my-lisp` CLI and LSP;
- Tauri project builds through Bun, Cargo and Tauri CLI adapters;
- the proof that `my-idea` can open and build its own checkout.

`my-idea` does not own:

- ecosystem observability or swarm control (`tauricode`);
- WSM language semantics (`my-lisp`);
- compiler semantics or backend policy (`cml`);
- an independent WSM parser, evaluator or diagnostic engine;
- arbitrary shell execution disguised as a build command.

## Honest compiler boundary

The first WSM adapter may expose operations already supported by the
authoritative tools, such as Run, Check and LSP diagnostics.

`cml` does not currently expose a stable, general-purpose target-selecting CLI.
Therefore `my-idea` must not display a universal **Compile** action for CML
until CML ratifies an exact command, target, output and diagnostic contract.

The Tauri build adapter is orchestration, not a new compiler. It invokes fixed
argument arrays in a selected project root:

```text
bun install --frozen-lockfile
bun run check
bun run tauri dev
bun run tauri build
```

## Self-build definition

Self-build means:

1. open the `my-idea` source checkout in `my-idea`;
2. detect it as a Tauri project;
3. run the declared build profile;
4. stream stdout and stderr;
5. produce a real application bundle;
6. record source commit, tool versions, exit state, artifact path and digest.

It does not mean that an installed application secretly contains every build
tool. A source checkout and declared Rust/Bun/Tauri toolchain remain explicit
inputs.

## UI truth rule

```text
visible command = executable end-to-end path
```

Until the new adapters exist, incomplete Observatory and generic runtime
buttons are removed from the primary UI rather than represented as working
features. Dormant backend code is removed only after Tauricode ownership and
historical preservation have been checked.

## Consequences

- The earlier System Observatory vision remains a historical design document;
  it is superseded as `my-idea` product direction by this ADR.
- The distribution-first platform roadmap is secondary to the working desktop
  development loop.
- Web/PWA support may continue, but self-build is a native desktop capability.
- Interactive PTY terminals, visual form designers and plugin systems are
  deferred until the plain build-output loop is proven useful.

