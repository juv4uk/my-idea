# Bun migration — record

Migrated the JavaScript package manager and script runner from npm to
[Bun](https://bun.sh) 1.3.8, conservatively: Bun replaces npm as the
dependency installer and `package.json` script runner; `shadow-cljs`,
Cargo, and `wasm-pack` are untouched, and `package.json`'s `build`/`test`
scripts keep shelling out to `node` internally where full Bun-runtime
compatibility wasn't independently verified. See `AGENTS.md`'s
"JavaScript package manager: Bun" section for the day-to-day rules this
produced.

## Baseline (before migration, WSL2 + Guix, `guix shell -m manifest.scm`)

- `node --version`: v22.14.0
- `npm --version`: 10.9.2
- `rustc --version` / `cargo --version`: 1.85.1 (this repo's Rust build is
  independently blocked by an unrelated EPERM in `tauri-build`'s ACL
  permission-file generation — see `tasks.my`'s `IDEA-CARGO-CHECK`; not
  this migration's concern, and not fixed here)
- `npm install`: OK
- `npm run check` (`shadow-cljs compile app`): PASS (0 errors, 2
  pre-existing `:infer-warning`s, unrelated to this migration)
- `npm test`: 13/15 pass. 2 pre-existing failures: `tests/conformance.test.mjs`
  and "Standalone web artifact does not stack overflow on 100k list" both
  need build artifacts (`public/wasm/`, `my-idea-web.html`) this checkout
  hasn't produced — not caused by this migration.
- `npm run build`: **FAIL** (exit 1, no output) — pre-existing:
  `scripts/build.mjs`'s `spawnSync('wasm-pack', ...)` fails silently
  because `wasm-pack` isn't in `manifest.scm`. Not fixed as part of this
  migration (out of scope per the migration's own ground rules).
- `npm run tauri dev`: not attempted — already known-blocked by the
  `IDEA-CARGO-CHECK` EPERM issue, independent of this migration.

## What changed

- `package.json`: added `"packageManager": "bun@1.3.8"`. Scripts
  themselves unchanged (`build`/`test`/`benchmark` still say `node ...`
  internally — see AGENTS.md for why).
- `bun.lock` added (committed). `package-lock.json` **retained** — see
  "Why package-lock.json wasn't removed" below.
- `.github/workflows/ci.yml`, `.github/workflows/publish-release.yml`:
  `oven-sh/setup-bun@v2` (pinned `1.3.8`) added alongside the existing
  `actions/setup-node@v6` (kept — see above), `npm ci` →
  `bun install --frozen-lockfile`, `npm run X` → `bun run X`,
  `npx tauri build` → `bunx tauri build`.
- `scripts/release.sh`, `scripts/release.ps1`: `npm test`/`npm run
  check`/`npm run build` → `bun run` equivalents, prefixed with
  `bun install --frozen-lockfile`. The `npm`/`npm.cmd version` calls that
  bump `package.json` **and** `package-lock.json` together were left
  as-is — no direct Bun equivalent was verified safe to swap in this pass,
  and `package-lock.json` is still the file of record until it's removed.
- `src-tauri/tauri.conf.json`: `beforeDevCommand`/`beforeBuildCommand`
  `npm run dev`/`npm run build` → `bun run dev`/`bun run build`.
- `README.md`, `docs/testing.md`, `docs/benchmarks.md`: commands updated
  to `bun run ...` (all UK/DE translations included where present).
- `AGENTS.md`: new "JavaScript package manager: Bun" section (rules +
  the DrvFs `fchmod` workaround below), legacy npm `--no-bin-links` note
  kept for context.
- `manifest.scm`: documented that Bun isn't packaged in this Guix channel
  (`guix search bun` — nothing); `node`/`npm` stay in the manifest since
  `build`/`test` scripts still need `node`.

## Known issue: `bun install` crashes on this DrvFs mount

A plain `bun install` on `/mnt/c/GitHub/my-idea` fails partway through
with:

```
EPERM: Operation not permitted: failed to change lockfile permissions (fchmod)
```

— reproducible, not intermittent. Same root cause class as the earlier
`npm install --no-bin-links` DrvFs issue (documented in AGENTS.md): DrvFs
doesn't support the permission-bit operation Bun's lockfile write needs,
this time `fchmod` on the just-written `bun.lock` rather than npm's
bin-symlink step. Package resolution and `node_modules` population both
complete successfully before the crash — only the final lockfile
permission change fails.

**Workaround** (used to produce the committed `bun.lock`, and documented
in AGENTS.md for future regenerations):

```bash
mkdir -p ~/bun-lockgen-tmp && cp package.json ~/bun-lockgen-tmp/
cd ~/bun-lockgen-tmp && bun install        # native WSL fs — fchmod works
cp bun.lock /mnt/c/GitHub/my-idea/bun.lock

cd /mnt/c/GitHub/my-idea && bun install --frozen-lockfile  # never rewrites
                                                             # the lockfile,
                                                             # so it never
                                                             # hits fchmod
```

This is also the reason `--frozen-lockfile` is mandatory in CI/release,
not just a reproducibility nicety here — a bare `bun install` on this
filesystem would fail outright.

## Validation results

| Check | Result |
|---|---|
| `bun install --frozen-lockfile` (after the lockfile workaround) | PASS — 157 packages |
| `bun run check` | PASS — same 2 pre-existing warnings as `npm run check` |
| `bun run test` | Same as baseline: 13/15 pass, same 2 pre-existing failures |
| `bun run build` | Same pre-existing failure as `npm run build` (missing `wasm-pack`), no new regression |
| `bun run wasm` | Same pre-existing failure as `npm run wasm` (missing `wasm-pack`), no new regression |
| `bun run tauri dev` / `bun run tauri build` | Not reachable — blocked by the independent, pre-existing `IDEA-CARGO-CHECK` Rust build EPERM (see `tasks.my`) |

`bun run test` producing byte-identical pass/fail counts and the same
specific failures as `npm test` is why the test script's runner
(`node --test` vs `bun test`) was **not** swapped — `bun run test`
(Bun as the *invoker* of the existing Node-based script) was verified;
`bun test` (Bun's own, different test runner) was not directly compared
against it, per the migration's own conservative rule.

## Why `package-lock.json` wasn't removed

The completion criteria for removing it require `bun run tauri dev` and
`bun run tauri build` to pass — both are blocked by the pre-existing,
unrelated Rust-side EPERM (`tasks.my`'s `IDEA-CARGO-CHECK`). Removing
`package-lock.json` now would be a bigger bet than this migration's own
"don't break the project" priority allows, given the Tauri build itself
can't be verified end-to-end yet. Revisit once that blocker clears.

## Why the Guix migration is partial

Bun has no package in the Guix channel pinned by `../my-lisp/channels.scm`
(`guix search bun` returns nothing). Installed via the official installer
script instead, `~/.bun/bin` on `$PATH`. Packaging Bun for Guix — either
upstream or a local package definition — is real, separate follow-up
work, not blocking this migration per its own rules (§15: "Не робити
Guix-міграцію блокуючою умовою для першого Bun-коміту").
