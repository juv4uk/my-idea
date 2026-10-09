# Compiler Artifact → Tauri Self-Build Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the #12 CML artifact a fresh, hash-verified, provenance-bearing required input to the self-build Tauri stage.

**Architecture:** Preserve the existing generic compiler path, add an explicit-output compiler API for self-build, generate deterministic compiler-stage evidence, and make the self-build Tauri wrapper enable a Cargo build gate that re-verifies hashes/revision before `tauri_build::build()`. Normal Tauri builds remain unchanged unless the self-build gate is explicitly enabled.

**Tech Stack:** Rust 2021, existing CompilerBridge/CompilerBuildAdapter, sha2, Serde/serde_json, Tauri build.rs, Node/Bun orchestration.

**Spec:** `docs/superpowers/specs/2026-09-18-compiler-artifact-tauri-self-build-design.md`

## Global Constraints

- Exact self-build target: `x86_64-linux`.
- Exact source path: `self-build/my-idea.lisp`.
- Exact compiler artifact: `target/self-build/my-idea-x86_64-linux`.
- CML remains semantic/compiler authority; orchestration never interprets Lisp.
- Freshness is SHA/provenance based, not mtime-only.
- Normal Tauri builds remain unchanged unless `MY_IDEA_REQUIRE_COMPILER_STAGE=1`.
- #14 generation witness is out of scope.
- Do not modify CML PR #133 or Help API PR #55.

---

### Task 1: Explicit compiler output path

**Files:**
- Modify: `src-tauri/src/compiler_bridge.rs`
- Modify: `src-tauri/src/compiler_build_adapter.rs`
- Create: `src-tauri/tests/self_build_compiler_output_contract.rs`
- Create: `self-build/my-idea.lisp`

**Interfaces:**
- Produce `CompilerBridge::compile_observed_to(request, output_path)`.
- Produce `CompilerBuildAdapter::compile_project_to(request, output_path)`.
- Existing `compile_observed` / `compile_project` remain compatible.

- [ ] RED test imports/calls the explicit-output API and asserts the exact self-build artifact path.
- [ ] Run the focused test; expected compile failure because the API does not exist.
- [ ] Implement the explicit-output API by moving current compile body behind an output-path parameter.
- [ ] Keep current generic API delegating to its existing derived `target/my-idea/...` path.
- [ ] Add tracked `self-build/my-idea.lisp` using CML syntax already proven by the real-CML fixture.
- [ ] In CI with provisioned CML, prove the tracked source compiles to exactly `target/self-build/my-idea-x86_64-linux`.
- [ ] Commit.

---

### Task 2: Deterministic compiler-stage evidence

**Files:**
- Modify: `src-tauri/src/self_build.rs`
- Modify: `src-tauri/tests/self_build_plan_contract.rs`
- Create: `src-tauri/tests/self_build_compiler_stage_contract.rs`

**Interfaces:**
- Produce serializable/deserializable `CompilerStageEvidence`.
- Produce `prepare_compiler_stage(repo_root, cml_executable, plan)`.
- Produce `verify_compiler_stage(repo_root, plan, evidence)`.

- [ ] RED: missing evidence API and stale/missing artifact rejection.
- [ ] Add SHA-256 helper over file bytes.
- [ ] Compile through `CompilerBuildAdapter::compile_project_to`.
- [ ] Require compiler identity/revision and input revision to equal #12 plan provenance.
- [ ] Record source/artifact SHA-256 and only repo-relative logical paths.
- [ ] Verify current source/artifact hashes + repo HEAD + plan digest.
- [ ] Run pure stale/missing/source-changed/artifact-changed tests plus real-CML evidence test.
- [ ] Commit.

---

### Task 3: Reusable Cargo/Tauri build gate

**Files:**
- Create: `src-tauri/build_support.rs`
- Modify: `src-tauri/build.rs`
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/tests/self_build_tauri_gate_contract.rs`

**Interfaces:**
- Produce a pure `verify_compiler_stage_gate(repo_root, gate)`.
- `build.rs` activates it only for `MY_IDEA_REQUIRE_COMPILER_STAGE=1`.

- [ ] RED: gate accepts no/missing/stale evidence today.
- [ ] Add `sha2` to build-dependencies (same locked version).
- [ ] Implement hash + source-revision verification in shared build-support code.
- [ ] In `build.rs`, read required env fields and fail before `tauri_build::build()` when invalid.
- [ ] On success emit compiler-stage provenance with `cargo:rustc-env=...` and rerun-if-changed source/artifact.
- [ ] Prove ordinary gate-disabled builds remain accepted.
- [ ] Commit.

---

### Task 4: Self-build execution wrapper

**Files:**
- Create: `src-tauri/src/bin/self-build-stage.rs`
- Create: `scripts/self-build-tauri.mjs`
- Modify: `package.json`
- Modify: `src-tauri/src/self_build.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: Node/Rust contract files as appropriate.

**Interfaces:**
- Rust stage executable emits one JSON `CompilerStageEvidence`.
- Package script `self-build:tauri` performs compiler → frontend → Tauri.
- Internal Tauri command remains optional; #14 may consume the stable mechanism later.

- [ ] RED: no executable wrapper / no package script exists.
- [ ] Move production CML resolution into one reusable public mechanism and make current `find_cml` delegate to it.
- [ ] Implement Rust stage executable: discover plan → compile exact output → verify → emit JSON.
- [ ] Implement Node orchestration with no Lisp parsing.
- [ ] Run existing frontend build before Tauri, then invoke exactly `bun run tauri build` with strict gate env.
- [ ] Write deterministic success evidence only after Tauri exits 0.
- [ ] Commit.

---

### Task 5: Physical CI witness and integration

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: relevant self-build docs only.

**Interfaces:**
- CI proves real pinned CML → exact artifact/evidence → strict Tauri/Cargo stage.
- Keep full packaging/generation proof for #14.

- [ ] Add a bounded self-build witness using the already provisioned pinned CML.
- [ ] Prove missing artifact and changed source/artifact are rejected by the build gate.
- [ ] Prove compiler evidence contains exact CML revision plus source/artifact SHA-256.
- [ ] Run the strict Tauri/Cargo path far enough to execute `build.rs` under the gate; do not claim generation 1 yet.
- [ ] Verify all existing repo gates and real-CML witness remain GREEN.
- [ ] Update PR with exact RED commits/run IDs, mark ready, merge only exact GREEN head.
