# Help Index Design — IDE-HELP-INDEX-1 (#43)

## Goal

Build one offline-first, machine-readable documentation index for `my-idea`, initially covering sibling checkouts of `my-lisp`, `my-idea`, and `cml`, while preserving each repository as the authority for its own documentation.

## Existing architecture to reuse

`src-tauri/src/ecosystem/mod.rs` already resolves the `my-idea` repository root and its sibling repository root without a network dependency. `src-tauri/src/ecosystem/git.rs` already reads Git metadata. The help index extends this ecosystem layer instead of creating a second repository-discovery mechanism.

## Data model v0

Each indexed document exposes:

- `repo`
- `path`
- `title`
- `commit_sha`
- `language`
- `category`
- `status`
- `source_kind`

The index keeps document text internally for full-text search but does not copy the corpus into new canonical files.

## Source scope v0

Registered repositories:

1. `my-lisp`
2. `my-idea`
3. `cml`

For each repository, scan Markdown documentation under `docs/` plus the repository root `README.md` when present. Later repositories are added only through explicit bounded source registration.

## Provenance

Every indexed document carries the full Git `HEAD` SHA of the checkout it was read from. If a repository is missing or Git provenance cannot be established, documents from that repository are not presented as authoritative indexed results; the source is reported unavailable.

Duplicate human titles never merge. Resource identity is `(repo, path, commit_sha)`.

## Status classification

Classification is intentionally conservative:

- `historical`: path is under an `archive` directory or the document explicitly declares a superseded/historical state;
- `adr`: file name begins with `ADR-`;
- `experiment`: file/path explicitly contains `experiment` or `spike`;
- `current`: only explicit current markers such as `CURRENT.md` or an explicit `Status: current` declaration;
- `unknown`: everything else.

When signals conflict, stronger historical/superseded evidence wins over current. No heuristic may silently promote an unknown document to current.

## Language classification

Only explicit filename suffixes are trusted in v0 (`.uk.md`, `.en.md`, `.sa.md`). Unmarked Markdown is `unknown`; content-language guessing is out of scope.

## Category / source kind

`source_kind` is structural: `readme`, `adr`, or `markdown`.

`category` is path-derived and non-semantic: for `docs/<category>/...`, use that first directory; top-level `docs/*.md` is `docs`; root README is `root`. Semantic grouping belongs to later help-topic work, not this file scanner.

## Search

Case-insensitive substring search across title, repo/path, and body text. Results are deterministic and sorted by `(repo, path)`; v0 deliberately avoids a hidden ranking algorithm. An empty query returns the metadata index in deterministic order.

## Boundaries

- No GitHub API or network dependency in #43.
- No copy of other repositories' docs into `my-idea` as a new source of truth.
- No Lisp semantic identity mapping yet; that belongs to #44.
- No Help Browser/F1 UI yet; that belongs to #45.
- Missing/unknown metadata stays explicit rather than guessed.

## First executable witnesses

1. Searching `ABI` over fixtures finds a CML ABI document with exact repo/path/full SHA.
2. Searching `Canon` returns same-titled documents from two repos as distinct results.
3. Archived documentation is never reported as current.
4. Rebuilding the same fixture index twice yields identical ordered metadata/results.
5. Changing only a source SHA changes the returned provenance without changing document identity fields other than `commit_sha`.
