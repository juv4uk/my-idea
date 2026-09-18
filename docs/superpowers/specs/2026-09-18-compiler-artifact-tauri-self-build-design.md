# Compiler artifact → Tauri self-build boundary (#13)

Date: 2026-09-18
Status: implementation contract derived from #13 acceptance and merged #12.

## Goal

Turn the compiler stage described by `self-build-plan-v1` into a real required input of the **self-build** Tauri stage.

The honest pipeline is:

```text
tracked self-build/my-idea.lisp
  → authoritative cml-compile
  → target/self-build/my-idea-x86_64-linux
  → verified compiler-stage evidence
  → frontend build
  → Tauri/Cargo build with compiler-stage gate enabled
```

This does not claim that CML compiles the Tauri GUI. CML owns the project-language stage; Tauri/Cargo remain the GUI/platform bootstrap mechanism.

## Scope boundary

- #13 executes and verifies one compiler stage, then feeds its evidence into the Tauri build.
- #14 owns the generation-0 → generation-1 → generation-2 witness.
- CML diagnostic spans remain upstream CML PR #133.
- Help API PR #55 remains unrelated.

Normal platform/release Tauri builds remain available. The strict compiler-stage requirement is enabled only by the self-build wrapper, because `self-build-plan-v1` currently has executable evidence only for target `x86_64-linux`.

## Canonical project-language input

Track:

`self-build/my-idea.lisp`

It is a deliberately small CML-supported project-language bootstrap witness, not a claim that the GUI is authored in Lisp. The first content uses syntax already physically proven by the pinned CML integration witness.

## Exact output path

The self-build compiler stage must produce exactly:

`target/self-build/my-idea-x86_64-linux`

The existing generic compiler path `target/my-idea/<stem>-<target>` remains unchanged for ordinary IDE Compile operations.

Therefore `CompilerBridge` gains an explicit-output entry point while preserving the existing API as a compatibility wrapper.

## Compiler-stage evidence

Schema: `self-build-compiler-stage-v1`.

Evidence records:

- self-build plan digest;
- source repository revision;
- source relative path;
- source SHA-256;
- compiler identity;
- compiler exact Git revision;
- compiler target;
- artifact relative path;
- artifact SHA-256.

Evidence contains no timestamp or temporary path.

Preparation succeeds only if:

- plan target is `x86_64-linux`;
- source exists and is in the clean source revision described by the plan;
- compiler identity/revision match the plan;
- CML produces the exact planned output path;
- source and artifact SHA-256 are readable.

## Freshness rule

Freshness is content/provenance based, not merely mtime based.

Verification rejects when:

- source or artifact is missing;
- current source SHA-256 differs from evidence;
- current artifact SHA-256 differs from evidence;
- current repository HEAD differs from evidence source revision;
- compiler revision, target, paths, or plan digest are malformed/mismatched.

A previously successful artifact therefore becomes stale immediately if the tracked source changes, even when filesystem mtimes are misleading.

## Tauri/Cargo gate

Self-build invokes the ordinary Tauri command with an explicit gate:

```text
bun run tauri build
```

The wrapper sets `MY_IDEA_REQUIRE_COMPILER_STAGE=1` and passes the verified compiler-stage evidence fields to Cargo.

`src-tauri/build.rs` checks the gate **before** `tauri_build::build()`:

- missing evidence → fail;
- missing artifact → fail;
- source/artifact hash mismatch → fail;
- source revision mismatch → fail.

On success it emits compiler-stage provenance through `cargo:rustc-env=...`, making the compiler output a physical build input rather than a pre-build side effect.

Without `MY_IDEA_REQUIRE_COMPILER_STAGE=1`, normal Tauri builds retain their current behavior.

## Execution wrapper

A small Rust self-build stage executable uses the production resolver and `CompilerBuildAdapter`; it does not implement Lisp semantics.

A Node/Bun orchestration script may only:

1. launch the Rust compiler-stage executable;
2. parse its machine-readable evidence;
3. run the existing frontend build;
4. run `bun run tauri build` with the compiler-stage gate environment;
5. write a deterministic Tauri-stage evidence record on success.

It must not parse or interpret Lisp.

## RED → GREEN evidence

1. RED: explicit planned compiler output path cannot be requested today.
2. GREEN: pinned real CML writes the exact `target/self-build/...` artifact without changing generic compile behavior.
3. RED: missing/stale compiler-stage evidence is currently not rejected by a Tauri build gate.
4. GREEN: reusable build-gate verifier rejects missing/source-changed/artifact-changed evidence.
5. GREEN: real pinned CML produces evidence containing source/artifact SHA-256 and exact compiler revision.
6. GREEN: self-build wrapper reaches the Tauri/Cargo build only with verified evidence; ordinary Tauri build remains unchanged.
7. Existing my-lisp single-channel, compiler bridge, real-CML and normal repo gates remain GREEN.
