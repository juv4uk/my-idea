# Help Index Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an offline-first, deterministic documentation index for `my-lisp`, `my-idea`, and `cml` with exact provenance, conservative status classification, and full-text lookup.

**Architecture:** Extend the existing `src-tauri/src/ecosystem` layer rather than inventing a second repository-discovery path. A new `documentation` module scans registered sibling repositories, keeps Markdown bodies internal for search, and exposes serializable metadata records/results; Git provenance comes from a reusable full-HEAD helper in `ecosystem/git.rs`.

**Tech Stack:** Rust stdlib, Serde, existing Tauri/Rust test harness; no new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-16-help-index-design.md`

## Global Constraints

- No network/GitHub API dependency in #43.
- Initial registered repos are exactly `my-lisp`, `my-idea`, and `cml`.
- Scan `docs/**/*.md` plus root `README.md` only.
- Resource identity remains `(repo, path, commit_sha)`; duplicate titles never merge.
- Default status is `unknown`; never infer `current` without an explicit marker.
- Language is explicit filename metadata only (`.uk.md`, `.en.md`, `.sa.md`), otherwise `unknown`.
- Search order is deterministic `(repo, path)`; no hidden ranking.

---

### Task 1: RED contract for provenance, status, duplicates, and deterministic search

**Files:**
- Create: `src-tauri/tests/documentation_index_contract.rs`

**Interfaces:**
- Consumes: future `my_idea_lib::ecosystem::documentation::{DocumentationIndex, DocumentationSource, DocumentStatus}`.
- Produces: executable contract for all #43 acceptance rules before implementation exists.

- [ ] **Step 1: Write the failing fixture-based tests**

Use temp directories with explicit source SHAs so the tests do not shell out to Git. The test API is:

```rust
let sources = vec![
    DocumentationSource::new("cml", cml_root.clone(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
    DocumentationSource::new("my-lisp", my_lisp_root.clone(), "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
];
let index = DocumentationIndex::build(&sources);
let hits = index.search("ABI");
```

Required assertions:

```rust
assert_eq!(hits[0].repo, "cml");
assert_eq!(hits[0].path, "docs/abi.md");
assert_eq!(hits[0].commit_sha, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
assert_eq!(index.search("Canon").len(), 2);
assert_ne!(index.search("Canon")[0].repo, index.search("Canon")[1].repo);
assert_eq!(historical.status, DocumentStatus::Historical);
assert_ne!(historical.status, DocumentStatus::Current);
assert_eq!(index.records(), DocumentationIndex::build(&sources).records());
```

Also assert `.uk.md -> "uk"`, unmarked `.md -> "unknown"`, `ADR-* -> Adr`, `CURRENT.md -> Current`, and empty search returns all records in `(repo,path)` order.

- [ ] **Step 2: Run the focused test and verify RED**

Run:

```bash
cd src-tauri
cargo test --test documentation_index_contract
```

Expected: compile failure because the `documentation` module/types do not exist yet.

- [ ] **Step 3: Commit RED witness**

```bash
git add src-tauri/tests/documentation_index_contract.rs
git commit -m "test(help): define documentation index contract"
```

---

### Task 2: Implement the documentation scanner and search model

**Files:**
- Create: `src-tauri/src/ecosystem/documentation.rs`
- Modify: `src-tauri/src/ecosystem/mod.rs`

**Interfaces:**
- Produces:

```rust
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum DocumentStatus { Current, Historical, Adr, Experiment, Unknown }

#[derive(Debug, Clone)]
pub struct DocumentationSource {
    pub repo: String,
    pub root: PathBuf,
    pub commit_sha: String,
}

impl DocumentationSource {
    pub fn new(repo: impl Into<String>, root: PathBuf, commit_sha: impl Into<String>) -> Self;
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DocumentRecord {
    pub repo: String,
    pub path: String,
    pub title: String,
    pub commit_sha: String,
    pub language: String,
    pub category: String,
    pub status: DocumentStatus,
    pub source_kind: String,
}

pub struct DocumentationIndex { /* records + internal bodies */ }

impl DocumentationIndex {
    pub fn build(sources: &[DocumentationSource]) -> Self;
    pub fn records(&self) -> Vec<DocumentRecord>;
    pub fn search(&self, query: &str) -> Vec<DocumentRecord>;
}
```

- [ ] **Step 1: Add module export**

In `ecosystem/mod.rs` add:

```rust
pub mod documentation;
```

- [ ] **Step 2: Implement bounded recursive Markdown discovery**

`DocumentationIndex::build` must:

1. include `<root>/README.md` when it is a file;
2. recursively walk `<root>/docs` using `std::fs::read_dir`;
3. include only regular files ending in `.md`;
4. use slash-normalized paths relative to the repo root;
5. sort candidate paths before reading them;
6. skip unreadable files rather than inventing records.

- [ ] **Step 3: Implement title/language/category/status/source-kind extraction**

Title:

```rust
fn title_from_markdown(body: &str, path: &Path) -> String {
    body.lines()
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|title| !title.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| fallback_title(path))
}
```

Status precedence:

```text
archive/superseded/historical -> Historical
ADR-*                         -> Adr
experiment/spike              -> Experiment
CURRENT.md or explicit current-> Current
otherwise                     -> Unknown
```

Explicit status recognition is bounded to case-insensitive lines beginning with `status:` or `superseded-by:`/`supersedes:`; do not use prose sentiment.

- [ ] **Step 4: Implement deterministic search**

Store a lowercase searchable string per record:

```rust
format!("{}\n{}\n{}\n{}", record.repo, record.path, record.title, body).to_lowercase()
```

`search("")` returns all metadata records. Non-empty search uses case-insensitive substring match. Always return records in the same sorted `(repo,path)` order as `records()`.

- [ ] **Step 5: Run the focused contract**

```bash
cd src-tauri
cargo test --test documentation_index_contract
```

Expected: PASS.

- [ ] **Step 6: Commit GREEN scanner**

```bash
git add src-tauri/src/ecosystem/documentation.rs src-tauri/src/ecosystem/mod.rs
 git commit -m "feat(help): add deterministic documentation index"
```

---

### Task 3: Wire real sibling repositories and exact Git provenance

**Files:**
- Modify: `src-tauri/src/ecosystem/git.rs`
- Modify: `src-tauri/src/ecosystem/mod.rs`
- Modify: `src-tauri/tests/documentation_index_contract.rs`

**Interfaces:**
- Produces:

```rust
pub(crate) fn repo_head_sha(repo: &Path) -> Option<String>;
pub fn documentation_index() -> documentation::DocumentationIndex;
pub fn documentation_search(query: &str) -> Vec<documentation::DocumentRecord>;
```

- [ ] **Step 1: Add a RED production-source test seam**

Extend the contract with a helper-level test for `DocumentationSource` provenance and missing source behavior: a source with an empty/nonexistent root yields no records, never records with guessed SHA. Keep fixture source SHAs full-length.

- [ ] **Step 2: Expose full HEAD SHA from existing Git helper**

In `ecosystem/git.rs` add:

```rust
pub(crate) fn repo_head_sha(repo: &Path) -> Option<String> {
    git_output(repo, &["rev-parse", "HEAD"])
}
```

Do not change existing short-SHA `RepoInfo` behavior.

- [ ] **Step 3: Reuse `siblings_root()` for registered v0 sources**

In `ecosystem/mod.rs` add:

```rust
pub fn documentation_index() -> documentation::DocumentationIndex {
    let Some(root) = siblings_root() else {
        return documentation::DocumentationIndex::default();
    };
    let mut sources = Vec::new();
    for repo in ["my-lisp", "my-idea", "cml"] {
        let path = root.join(repo);
        if let Some(sha) = git::repo_head_sha(&path) {
            sources.push(documentation::DocumentationSource::new(repo, path, sha));
        }
    }
    documentation::DocumentationIndex::build(&sources)
}

pub fn documentation_search(query: &str) -> Vec<documentation::DocumentRecord> {
    documentation_index().search(query)
}
```

`DocumentationIndex` therefore needs `Default` returning an empty index.

- [ ] **Step 4: Run focused and full Rust verification**

```bash
cd src-tauri
cargo test --test documentation_index_contract
cargo test
cargo check
```

Expected: all PASS.

- [ ] **Step 5: Run repository gates**

From repository root:

```bash
bun run test
bun run check
```

Expected: all PASS.

- [ ] **Step 6: Commit production wiring**

```bash
git add src-tauri/src/ecosystem/git.rs src-tauri/src/ecosystem/mod.rs src-tauri/tests/documentation_index_contract.rs
git commit -m "feat(help): index sibling repo documentation with provenance"
```

---

## Self-review

- Spec coverage: provenance, three initial repos, status fail-closed, language fail-closed, duplicate-title separation, deterministic rebuild/search, full-text lookup, and no copied corpus are all covered by Tasks 1–3.
- Placeholder scan: no TBD/TODO or unspecified implementation steps remain.
- Type consistency: `DocumentationSource`, `DocumentationIndex`, `DocumentRecord`, and `DocumentStatus` names/signatures are identical across tasks.
- Scope boundary: no Lisp Help API (#44), F1/UI (#45), plugin docs (#46), or CHM/export (#47) is implemented here.
