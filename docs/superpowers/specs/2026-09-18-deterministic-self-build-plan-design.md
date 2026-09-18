# Deterministic self-build plan design (#12)

Date: 2026-09-18
Status: approved by project owner in chat; bounded implementation of the accepted self-building IDE direction.

## Goal

Before `my-idea` executes any self-build, it must be able to emit one deterministic, inspectable plan for building its next generation. This slice plans only. Execution belongs to #13 and generation proof to #14.

The honest claim remains:

```text
self-building using our project compiler/CML + Tauri platform mechanism
```

It is not full self-hosting.

## Authority boundaries

- `my-lisp` owns Lisp semantics. #12 only records its exact pinned revision.
- CML is the authoritative project compiler boundary. #12 records its exact executable identity/revision and a fixed supported target.
- Rust/Tauri code owns orchestration/provenance transport only.
- Tauri is explicitly recorded as a bootstrap/platform mechanism.
- #12 must not execute CML, Bun, Cargo, Tauri, or a build artifact.

## First supported target

The first plan target is exactly `x86_64-linux`, because that is the real compiler target currently proven through `CompilerBridge` / `cml-compile`.

No synthetic multi-platform plan is emitted. Other targets require later executable evidence.

## Inputs

A plan is constructed from explicit, already-resolved provenance:

- source repository revision: full 40-hex Git HEAD;
- source tree state: clean only;
- `my-lisp` revision: exact `MY_LISP_PINNED_SHA` from the #49 single-channel pin;
- compiler executable identity: `cml-compile`;
- compiler revision: full 40-hex Git SHA;
- compiler target: `x86_64-linux`;
- Rust toolchain identity: explicit `rustc` and `cargo` version strings;
- Bun version;
- Tauri CLI version.

Missing, empty, `unavailable`, malformed SHA, dirty source tree, or unsupported target fails closed.

The pure plan builder consumes a `SelfBuildInputs` value. Environment/process discovery is a separate mechanism function so deterministic tests do not depend on ambient machine state.

## Plan schema

Schema version: `self-build-plan-v1`.

The normalized plan contains:

- schema version;
- source: repository-relative root `.`, source revision, clean=true;
- language runtime: `my-lisp` revision;
- compiler: executable `cml-compile`, exact revision, target `x86_64-linux`;
- toolchain: rustc, cargo, bun, tauri version strings;
- ordered stages;
- digest.

Ordered stages are:

1. `compiler`: authoritative CML project-language stage.
2. `frontend`: WASM + ClojureScript/frontend build stage using the repository's declared build command.
3. `platform`: Tauri build/bundle stage, explicitly marked `bootstrap-platform=true`.

Commands are represented as executable plus argv arrays, never shell command strings.

For v1 the plan describes the already-declared mechanisms:

```text
compiler:
  cml-compile x86-elf <canonical project source> <compiler artifact>

frontend:
  bun run build

platform:
  bun run tauri build
```

The plan uses repository-relative logical paths only. It never contains timestamps, random IDs, temporary directories, absolute checkout paths, usernames, or host-specific cache paths.

## Project source and artifact logical paths

The self-build plan names logical paths, not yet-executed files:

- canonical project source: `self-build/my-idea.lisp`;
- compiler artifact: `target/self-build/my-idea-x86_64-linux`;
- platform output root: `src-tauri/target/release/bundle`.

#12 does not claim the canonical project source already compiles the whole GUI. #13 is responsible for making the compiler artifact a required Tauri pre-build input. The v1 plan makes this future dependency explicit and inspectable without pretending it is executed today.

## Deterministic normalization and digest

The digest is SHA-256 over canonical JSON of the plan body without the digest field.

Canonicalization rules:

- struct field order is fixed by the Rust serialization type;
- stage order is fixed;
- argv order is fixed;
- no maps with unstable iteration order;
- no optional environment-derived fields beyond explicit inputs;
- UTF-8 JSON produced by `serde_json::to_vec`;
- digest encoded as lowercase 64-character hex.

Same inputs must produce byte-identical normalized JSON and the same digest. Changing any recorded revision/version/target must change the digest.

## API boundary

Pure Rust:

```rust
pub struct SelfBuildInputs { ... }
pub struct SelfBuildPlan { ... }

pub fn build_self_build_plan(inputs: SelfBuildInputs) -> Result<SelfBuildPlan, SelfBuildPlanError>;
pub fn canonical_plan_json(plan: &SelfBuildPlan) -> Result<String, SelfBuildPlanError>;
```

Mechanism discovery:

```rust
pub fn discover_self_build_inputs(
    repo_root: &Path,
    cml_executable: &Path,
) -> Result<SelfBuildInputs, SelfBuildPlanError>;
```

Tauri command:

```text
self_build_plan
```

It discovers inputs using the current workspace/repository and the existing production CML resolver, then returns the serializable plan. It never starts a build.

## Failure model

Fail closed when:

- Git HEAD cannot be resolved to a full SHA;
- source tree is dirty;
- `MY_LISP_PINNED_SHA` is absent/malformed;
- CML executable is missing;
- CML revision cannot be resolved exactly;
- target differs from `x86_64-linux`;
- rustc/cargo/Bun/Tauri version cannot be captured;
- canonical serialization/digest construction fails.

Errors identify which provenance input is missing; they do not silently substitute `unknown` or `unavailable`.

## Executable evidence

RED first:

1. no self-build plan API exists;
2. same inputs must yield identical normalized JSON/digest;
3. one changed SHA/version must change digest;
4. dirty/missing provenance must fail;
5. exact stage order must be CML → frontend → Tauri;
6. Tauri stage must explicitly identify itself as bootstrap/platform mechanism;
7. building a plan must not launch any external build command.

Existing CI, my-lisp single-channel, real-CML, JS/Rust/Tauri gates must remain GREEN.

## Coordination

- #12 owns deterministic planning only.
- #13 will consume this schema and require a fresh compiler artifact before Tauri build.
- #14 will record two plan/execution transitions for generation 0→1→2.
- `cml#132` owns source-location diagnostic transport and is independent of this plan.
- Help API #55 is unrelated and must not be modified.
