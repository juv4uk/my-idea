# Repository Analysis: my-idea

## Overview

my-idea is a lightweight cross-platform IDE for language development, built with Tauri (Rust backend) and ClojureScript (frontend). It targets the my-lisp ecosystem — a small Lisp language that grows itself — providing an embedded editor, evaluator, and ecosystem dashboard.

## Tech Stack

| Layer | Technology | Role |
|-------|-----------|------|
| Backend | Rust + Tauri v2 | Native IPC, file I/O, oracle/swarm queries |
| Frontend | ClojureScript + shadow-cljs | UI, editor, state management |
| Editor | CodeMirror 6 | Syntax highlighting, code editing |
| WASM | my-lisp-wasm (wasm-pack) | Web/PWA execution engine |
| JS runtime | Bun 1.3.8 | Package manager, build scripts |
| Package manager | Guix (channels.scm + manifest.scm) | Reproducible toolchain |
| Testing | node:test + Playwright | Frontend tests, browser E2E |

## Directory Structure

```
my-idea/
├── src-cljs/my_idea/     # ClojureScript frontend (10 files)
├── src-tauri/src/         # Rust backend (4 files)
├── tests/                 # Node + Playwright tests (3 files)
├── docs/                  # Documentation (14 files)
├── scripts/               # Build/release helpers
├── public/                # Static assets, WASM, CSS, PWA
├── external/my-lisp/      # Git submodule (my-lisp runtime)
├── benchmarks/            # my-lisp benchmark programs (.my)
├── .github/workflows/     # CI + release pipelines
├── manifest.scm           # Guix toolchain manifest
├── package.json           # JS dependencies + scripts
├── shadow-cljs.edn        # ClojureScript build config
└── AGENTS.md              # Agent instructions
```

## Source Files

### ClojureScript (src-cljs/my_idea/)

| File | Lines | Responsibility |
|------|-------|----------------|
| `commands.cljs` | 266 | Action/command layer (IPC calls, state mutation) |
| `core.cljs` | 94 | Render function, init entry point |
| `eco_view.cljs` | 128 | Ecosystem panel HTML builders (pure) |
| `editor.cljs` | — | CodeMirror 6 integration |
| `i18n.cljs` | — | Trilingual UI (uk/de/en) |
| `preview.cljs` | — | Markdown/Mermaid preview rendering |
| `state.cljs` | 22 | App state atom + active-doc helper |
| `util.cljs` | — | Utility functions (escape, next-value) |
| `wasm.cljs` | 85 | WASM engine loader and bindings |
| `workspace.cljs` | — | File tree, document management, IPC bridge |

### Rust (src-tauri/src/)

| File | Responsibility |
|------|----------------|
| `lib.rs` | Tauri commands, ecosystem status, language adapter |
| `main.rs` | Entry point |
| `oracle.rs` | TCP oracle client (my-lisp at 127.0.0.1:9999) |
| `swarm.rs` | Swarm-node status client (127.0.0.1:9104) |

## Test Coverage

| File | Tests | Covers |
|------|-------|--------|
| `conformance.test.mjs` | 19 | my-lisp fixtures via WASM, exact arithmetic, stack safety |
| `smoke.test.mjs` | 14 | Static wiring, PWA, Tauri commands, 100k list stack |
| `eco-panel.test.mjs` | 4 | Ecosystem panel render, repo-summary, evidence-matrix, fixture drill-down |
| Rust unit tests | 1 | Native adapter bootstrap library |
| **Total** | **38** | |

## Documentation

| File | Topic |
|------|-------|
| `README.md` | Project overview, trilingual |
| `AGENTS.md` | Agent instructions, Bun, Guix, DrvFs |
| `testing.md` | Test results (95 tests: 53 Rust + 42 Web/JS) |
| `bun-migration.md` | npm → Bun migration |
| `source-files.md` | .my extension conventions |
| `language-core.md` | my-lisp language reference |
| `platform-roadmap.md` | Platform targets |
| `system-observatory-*.md` | Vision and status docs |
| `versioning.md` | Version policy |
| `release-assets.md` | Release artifact docs |
| `benchmarks.md` | Performance benchmarks |
| `android-release.md` | Android build |
| `windows-arm64.md` | Windows ARM64 |
| `quote-tutorial.md` | Quote syntax tutorial |

## Build System

### Guix (reproducible toolchain)

- `manifest.scm`: rust, rust:cargo, nss-certs, node, openjdk, webkitgtk-for-gtk3, gtk+, libappindicator
- `channels.scm` (from ../my-lisp): pins Guix commit `5375f33` (rust 1.93.0)
- Entry: `guix time-machine -C ../my-lisp/channels.scm -- shell -m manifest.scm`

### Bun + shadow-cljs

- `bun run build`: WASM pack → shadow-cljs release → dist/ assembly
- `bun run test`: node --test + shadow-cljs compile test
- `scripts/make-portable-web.mjs`: generates standalone `my-idea-web.html` (1.6MB with embedded WASM)

### Release

- `scripts/release.sh <version>`: bumps versions, runs tests, builds, tags, pushes
- GitHub Actions: builds AppImage, deb, rpm, dmg, exe, msi, web.html
- Current release: v0.12.0

## Swarm Integration

- **Node ID**: `my-idea-1` (epoch 5)
- **Swarm-node port**: 9104
- **Event log**: `/home/my-idea/.swarm-node/my-idea-1/events.log`
- **Tasks**: IDEA-* tasks (CARGO-CHECK, CLJS-SPLIT, ECO-THEME, BUN-TAURI, etc.)
- **Ecosystem panel**: shows repo status, evidence matrix, compatibility lens (branch@sha)
- **Oracle integration**: TCP query to my-lisp at 127.0.0.1:9999
- **Swarm status button**: queries local swarm-node for live status

### Completed Swarm Tasks (by big-pickle-1)

- CML-C-BACKEND-ERROR-HANDLING
- ARCH-RECOVERY-REVIEW-IDE
- IDEA-ECO-THEME-AUDIT
- SWARM-GEN0-OWNERSHIP-SWEEP
- FPGA-GC-DESIGN-DOC
- IDEA-CORE-CLJS-SPLIT-COMMANDS
- IDEA-PLAYWRIGHT-ECO-COVERAGE
- IDEA-BUN-TAURI-VALIDATION
- IDEA-RUN-TESTS-BUTTON
- IDEA-COMPAT-LENS
