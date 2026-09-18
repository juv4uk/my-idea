# Deterministic Self-Build Plan Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an inspectable, deterministic, fail-closed self-build plan for `my-idea` without executing a build.

**Architecture:** A pure Rust plan builder owns schema validation, canonical serialization, and SHA-256 digesting from explicit provenance inputs. A separate discovery function gathers source/toolchain/compiler provenance from the current checkout, and a Tauri command exposes the resulting plan. #12 stops at planning; #13 executes the compiler/Tauri chain.

**Tech Stack:** Rust 2021, Serde/serde_json, sha2 0.10, std::process::Command, Tauri v2.

**Spec:** `docs/superpowers/specs/2026-09-18-deterministic-self-build-plan-design.md`

## Global Constraints

- First supported compiler target is exactly `x86_64-linux`.
- Stage order is exactly CML compiler → frontend/WASM/CLJS → Tauri platform build.
- Tauri is explicitly recorded as a bootstrap/platform mechanism.
- The plan never launches CML, Bun build, Cargo build, Tauri build, or produced artifacts.
- Missing/dirty/unavailable provenance fails closed.
- Plan data contains repository-relative logical paths only; no timestamps/temp paths/user paths.
- Digest is lowercase SHA-256 over the canonical JSON plan body without the digest field.
- #12 does not implement #13 execution or #14 generation witnesses.
- Do not touch active `cml#132` diagnostic work or Help API #55.

---

### Task 1: Establish the RED deterministic-plan contract

**Files:**
- Create: `src-tauri/tests/self_build_plan_contract.rs`

**Interfaces:**
- Consumes: future `my_idea_lib::self_build::{build_self_build_plan, canonical_plan_json, SelfBuildInputs}`.
- Produces: executable acceptance contract for Tasks 2–4.

- [ ] **Step 1: Write the failing integration test**

Create `src-tauri/tests/self_build_plan_contract.rs` with helpers that construct fixed inputs:

```rust
use my_idea_lib::self_build::{
    build_self_build_plan, canonical_plan_json, SelfBuildInputs,
};

fn inputs() -> SelfBuildInputs {
    SelfBuildInputs::new(
        "1111111111111111111111111111111111111111",
        true,
        "2222222222222222222222222222222222222222",
        "cml-compile",
        "3333333333333333333333333333333333333333",
        "x86_64-linux",
        "rustc 1.93.0",
        "cargo 1.93.0",
        "1.3.8",
        "2.11.4",
    )
}
```

Add tests that assert:

```rust
#[test]
fn same_inputs_produce_byte_identical_plan_and_digest() {
    let first = build_self_build_plan(inputs()).unwrap();
    let second = build_self_build_plan(inputs()).unwrap();
    assert_eq!(canonical_plan_json(&first).unwrap(), canonical_plan_json(&second).unwrap());
    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.digest().len(), 64);
}

#[test]
fn changing_compiler_revision_changes_digest() {
    let first = build_self_build_plan(inputs()).unwrap();
    let changed = SelfBuildInputs::new(
        "1111111111111111111111111111111111111111",
        true,
        "2222222222222222222222222222222222222222",
        "cml-compile",
        "4444444444444444444444444444444444444444",
        "x86_64-linux",
        "rustc 1.93.0",
        "cargo 1.93.0",
        "1.3.8",
        "2.11.4",
    );
    assert_ne!(first.digest(), build_self_build_plan(changed).unwrap().digest());
}

#[test]
fn plan_records_exact_stage_order_and_tauri_bootstrap_boundary() {
    let plan = build_self_build_plan(inputs()).unwrap();
    assert_eq!(plan.stage_names(), ["compiler", "frontend", "platform"]);
    assert!(plan.platform_stage().bootstrap_platform());
    assert_eq!(plan.compiler_target(), "x86_64-linux");
}
```

Also test dirty source, malformed/missing SHA, `unavailable` compiler revision, and target `aarch64-linux` all return `Err`.

- [ ] **Step 2: Run the exact RED test**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test self_build_plan_contract
```

Expected: compile failure because `my_idea_lib::self_build` does not exist.

- [ ] **Step 3: Commit RED**

```bash
git add src-tauri/tests/self_build_plan_contract.rs
git commit -m "test(#12): RED define deterministic self-build plan contract"
```

---

### Task 2: Implement the pure deterministic plan model

**Files:**
- Create: `src-tauri/src/self_build.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Test: `src-tauri/tests/self_build_plan_contract.rs`

**Interfaces:**
- Produces:
  - `SelfBuildInputs::new(...) -> SelfBuildInputs`
  - `build_self_build_plan(SelfBuildInputs) -> Result<SelfBuildPlan, SelfBuildPlanError>`
  - `canonical_plan_json(&SelfBuildPlan) -> Result<String, SelfBuildPlanError>`
  - plan getters used by Task 1.

- [ ] **Step 1: Add SHA-256 dependency and module export**

In `src-tauri/Cargo.toml`:

```toml
sha2 = "0.10"
```

In `src-tauri/src/lib.rs` with the other public modules:

```rust
pub mod self_build;
```

- [ ] **Step 2: Implement explicit input and plan types**

In `src-tauri/src/self_build.rs`, define serializable structs with fixed field order:

```rust
use serde::Serialize;
use sha2::{Digest, Sha256};

pub const SELF_BUILD_SCHEMA: &str = "self-build-plan-v1";
pub const SELF_BUILD_TARGET: &str = "x86_64-linux";

#[derive(Clone, Debug)]
pub struct SelfBuildInputs {
    source_revision: String,
    source_clean: bool,
    my_lisp_revision: String,
    compiler: String,
    compiler_revision: String,
    compiler_target: String,
    rustc_version: String,
    cargo_version: String,
    bun_version: String,
    tauri_version: String,
}
```

Define body records `SourcePlan`, `LanguageRuntimePlan`, `CompilerPlan`, `ToolchainPlan`, `BuildStage`, `SelfBuildPlanBody`, and `SelfBuildPlan`. Stage commands are:

```rust
BuildStage::compiler(
    "cml-compile",
    ["x86-elf", "self-build/my-idea.lisp", "target/self-build/my-idea-x86_64-linux"],
)
BuildStage::frontend("bun", ["run", "build"])
BuildStage::platform("bun", ["run", "tauri", "build"], true)
```

- [ ] **Step 3: Implement fail-closed validation**

Add helpers:

```rust
fn require_sha(label: &str, value: &str) -> Result<(), SelfBuildPlanError> {
    if value.len() != 40 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(SelfBuildPlanError::InvalidProvenance(format!(
            "{label} must be a full 40-hex git revision"
        )));
    }
    Ok(())
}
```

Reject:
- `source_clean == false`;
- compiler name not exactly `cml-compile`;
- target not exactly `x86_64-linux`;
- empty or `unavailable` version/provenance strings.

- [ ] **Step 4: Implement canonical JSON and SHA-256**

Construct `SelfBuildPlanBody`, serialize it with:

```rust
let bytes = serde_json::to_vec(&body)
    .map_err(|error| SelfBuildPlanError::Serialization(error.to_string()))?;
let digest = format!("{:x}", Sha256::digest(&bytes));
```

Store the body fields plus digest in `SelfBuildPlan`. `canonical_plan_json` serializes the full plan using `serde_json::to_string`.

- [ ] **Step 5: Run the focused contract**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test self_build_plan_contract
```

Expected: all Task 1 tests PASS.

- [ ] **Step 6: Commit pure GREEN**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs src-tauri/src/self_build.rs src-tauri/tests/self_build_plan_contract.rs Cargo.lock
git commit -m "feat(#12): build deterministic self-build plans"
```

---

### Task 3: Add provenance discovery without build execution

**Files:**
- Modify: `src-tauri/src/self_build.rs`
- Modify: `src-tauri/src/compiler_bridge.rs`
- Test: `src-tauri/tests/self_build_plan_contract.rs`

**Interfaces:**
- Consumes:
  - `repl_process::my_lisp_pinned_sha() -> &'static str`
  - compiler provenance helper from `compiler_bridge`.
- Produces:
  - `discover_self_build_inputs(repo_root: &Path, cml_executable: &Path) -> Result<SelfBuildInputs, SelfBuildPlanError>`.

- [ ] **Step 1: Expose compiler provenance helpers crate-locally**

Change:

```rust
fn compiler_identity(...)
fn git_revision_for(...)
```

to:

```rust
pub(crate) fn compiler_identity(...)
pub(crate) fn git_revision_for(...)
```

No behavior change.

- [ ] **Step 2: Implement command probes that only inspect provenance**

Add in `self_build.rs`:

```rust
fn command_version(program: &str, args: &[&str]) -> Result<String, SelfBuildPlanError> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|error| SelfBuildPlanError::Discovery(format!(
            "{program} version probe failed: {error}"
        )))?;
    if !output.status.success() {
        return Err(SelfBuildPlanError::Discovery(format!(
            "{program} version probe returned non-zero"
        )));
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if value.is_empty() {
        return Err(SelfBuildPlanError::Discovery(format!(
            "{program} version probe was empty"
        )));
    }
    Ok(value)
}
```

Discovery runs only:
- `git rev-parse HEAD`;
- `git status --porcelain`;
- `rustc --version`;
- `cargo --version`;
- `bun --version`;
- `bun run tauri --version`.

It never invokes a build command.

- [ ] **Step 3: Build explicit inputs**

Use:
- source HEAD from repo root;
- dirty state from `git status --porcelain`;
- `my_lisp_pinned_sha()`;
- `compiler_identity(cml_executable)`;
- `git_revision_for(cml_executable)`;
- fixed target `SELF_BUILD_TARGET`;
- captured tool versions.

Reject CML revision `unavailable` through the Task 2 validator.

- [ ] **Step 4: Add a discovery smoke witness**

Add a test guarded by availability of the actual source checkout/CML environment only if CI already provisions it; otherwise keep deterministic discovery helper unit tests inside `self_build.rs` for parsing/status behavior. Do not add network fetches or a second compiler resolver.

- [ ] **Step 5: Run focused + existing compiler tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test self_build_plan_contract
cargo test --manifest-path src-tauri/Cargo.toml --test compiler_bridge_contract
cargo test --manifest-path src-tauri/Cargo.toml --test real_cml_compiler_contract
```

Expected: PASS, with the existing real-CML test using its current CI provisioning contract.

- [ ] **Step 6: Commit discovery**

```bash
git add src-tauri/src/self_build.rs src-tauri/src/compiler_bridge.rs src-tauri/tests/self_build_plan_contract.rs
git commit -m "feat(#12): discover fail-closed self-build provenance"
```

---

### Task 4: Expose the plan through Tauri without executing it

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/self_build_plan_contract.rs`

**Interfaces:**
- Produces Tauri command `self_build_plan`.
- Reuses existing production `find_cml(&Path)`.

- [ ] **Step 1: Add the command**

Near other mechanism commands in `lib.rs`:

```rust
#[tauri::command]
fn self_build_plan(
    workspace: State<'_, Workspace>,
) -> Result<self_build::SelfBuildPlan, String> {
    let repo_root = root(&workspace)?;
    let cml = find_cml(&repo_root);
    let inputs = self_build::discover_self_build_inputs(&repo_root, &cml)
        .map_err(|error| error.to_string())?;
    self_build::build_self_build_plan(inputs)
        .map_err(|error| error.to_string())
}
```

Register `self_build_plan` in `tauri::generate_handler!`.

- [ ] **Step 2: Keep execution out of #12**

Do not call:
- `CompilerBridge::compile*`;
- `CompilerBuildAdapter::compile_project`;
- `ProcessService::start`;
- `bun run build`;
- `bun run tauri build`.

The returned object is data only.

- [ ] **Step 3: Run the Rust witness suite**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --tests
```

Expected: all existing and new Rust witnesses PASS.

- [ ] **Step 4: Commit Tauri exposure**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(#12): expose inspectable self-build plan"
```

---

### Task 5: Verify full repo gates and document the boundary

**Files:**
- Modify: `docs/IDE-IMPLEMENTATION-PLAN.md`
- Modify: `docs/ADR-003-SIMPLE-SELF-BUILDING-IDE.md`

**Interfaces:**
- Documents current reality: CML CLI boundary now exists; #12 plans only; #13 executes.

- [ ] **Step 1: Update stale CML/self-build wording**

In `docs/IDE-IMPLEMENTATION-PLAN.md`, replace the stale `cml | future stable compiler CLI | blocked` entry with the current pinned `cml-compile` boundary and note that self-build planning is separate from execution.

In ADR-003 add a dated implementation-status note rather than rewriting the historical decision: CML CLI is now ratified by executable evidence; #12 emits only a deterministic plan.

- [ ] **Step 2: Run full CI-equivalent gates**

```bash
python3 scripts/check-my-lisp-sync.py
bun install --frozen-lockfile
bun run build
bun run test
bun run check
cargo test --manifest-path src-tauri/Cargo.toml --tests
```

Expected: all pass. In GitHub CI, the pinned real-CML provenance lane must also remain GREEN.

- [ ] **Step 3: Verify no scope drift**

Diff must not change:
- Help API #55 files except incidental line movement with no semantic edit;
- CML upstream code;
- #13 execution wiring;
- #14 generation witness.

- [ ] **Step 4: Commit docs and open/ready PR**

```bash
git add docs/IDE-IMPLEMENTATION-PLAN.md docs/ADR-003-SIMPLE-SELF-BUILDING-IDE.md
git commit -m "docs(#12): record deterministic self-build planning boundary"
```

PR acceptance evidence must include:
- RED commit and exact failure;
- exact final head;
- canonical-plan/digest test;
- dirty/missing provenance failures;
- stage-order/Tauri bootstrap witness;
- full CI run ID and real-CML provenance line.
