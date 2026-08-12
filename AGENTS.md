# AGENTS.md — my-idea

## Role

Observer/IDE layer for the four-repository ecosystem (`my-lisp`,
`fpga-lisp`, `cml`, `my-idea`). Reads the other repos' contracts and
`evidence/` directories to render ecosystem status ("System Observatory");
does not itself define language or hardware semantics.

## Environment model

Three layers, in order:

```
Windows
  └── WSL2
       └── GNU Guix
            └── project environment (manifest.scm)
```

Windows is the host and UI layer. WSL is the Linux execution layer. Guix
defines this repo's actual dependencies. Perform builds, tests, and
`shadow-cljs`/`cargo` work inside WSL/Guix, not with Windows-installed
compilers or runtimes — those aren't part of the reproducible environment.

## Repository location

This repo currently lives at `/mnt/c/GitHub/my-idea` (Windows filesystem,
mounted into WSL), not the WSL-native `~/...` filesystem — a deliberate
choice by the owner, not an oversight; don't relocate it into WSL-native
storage on your own initiative. The sibling layout other ecosystem tooling
(including this repo's own `ecosystem::git`/`ecosystem::evidence` modules)
discovers by convention is:

```
/mnt/c/GitHub/
├── my-lisp/
├── cml/
├── fpga-lisp/
└── my-idea/
```

## Entering the environment

```
wsl -u my-idea
cd /mnt/c/GitHub/my-idea
guix shell -m manifest.scm
```

`manifest.scm` pins the toolchain versions this repo expects (Rust, Node,
OpenJDK for shadow-cljs, Tauri's Linux runtime deps). Prefer a clean run
for verification, especially before recording evidence:

```
guix shell --pure -m manifest.scm -- <command>
```

`nss-certs` is in the manifest for TLS, but `SSL_CERT_DIR` isn't wired in
automatically — export it before `cargo`/`npm` network operations:

```
export SSL_CERT_DIR="$GUIX_ENVIRONMENT/etc/ssl/certs"
```

## Guix features worth using deliberately

Beyond `guix shell -m manifest.scm`, three commands matter most for this
repo's evidence work:

- `guix shell --pure -m manifest.scm -- <command>` — strips the ambient
  WSL `$PATH`, so a pass can't be quietly depending on something installed
  outside the declared environment. Use this, not a bare `guix shell`, when
  a result is going into an evidence file.
- `guix describe` — prints the exact Guix commit/channel state; worth
  capturing alongside a result if you want to reproduce the *environment*
  later, not just the code (`git` pins the code, this pins the toolchain).
- `guix time-machine -C channels.scm -- shell -m manifest.scm -- <command>`
  — reproduces a specific past Guix revision via a pinned `channels.scm`,
  for when "same manifest, same day" isn't precise enough.

An ecosystem-wide `channels.scm` (one level up, alongside all four repos)
would let evidence eventually record `(guix-revision ...)` next to
`(commit ...)` — not implemented yet, and not this repo's call alone: it'd
mean extending the shared `evidence/README.md` schema across all four
repos, not something to add unilaterally to `ecosystem::evidence`'s reader.
If you want this, raise it with whoever owns that schema (currently
my-lisp) rather than inventing a parallel field here.

## JavaScript package manager: Bun

This repo uses [Bun](https://bun.sh) (`packageManager: "bun@1.3.8"` in
`package.json`) as the JavaScript package manager and script runner —
not npm, yarn, or pnpm.

- Use `bun install`, not `npm install`. Don't create `package-lock.json`.
- Use `bun install --frozen-lockfile` in reproducible/CI contexts — never
  let a release or CI run silently change `bun.lock`.
- Use `bun run <script>` for `package.json` scripts (`bun run check`,
  `bun run test`, `bun run build`, `bun run tauri dev`, ...).
- Don't update `bun.lock` unless a dependency change is actually intended
  — an incidental `bun install` without `--frozen-lockfile` can drift it.
- Bun is the package manager and script runner, not a replacement for
  `shadow-cljs`, Cargo, or `wasm-pack` — don't try to make it compile
  ClojureScript, Rust, or WASM directly.
- `build` and `test` still shell out to `node` internally (`node
  scripts/build.mjs`; `node --test tests/*.test.mjs && shadow-cljs
  compile test`) — this is intentional, not a leftover. `bun run test`
  produces byte-identical results to the old `npm test` (verified:
  13 pass / 2 known pre-existing failures, same two). Swapping the
  underlying runner to `bun test` was evaluated and deliberately not
  done — do it only after directly comparing `node --test` vs `bun test`
  output for this suite, per the migration record in
  `docs/bun-migration.md`.
- Bun's own binary (`~/.bun/bin/bun` after the official install script;
  not currently packaged in this Guix channel — `guix search bun` finds
  nothing) needs `~/.bun/bin` on `$PATH`.

### Known DrvFs limitation: `bun install`'s lockfile write

On this repo's filesystem mount (`/mnt/c/GitHub/my-idea`, DrvFs — see
"Repository location" above), a plain `bun install` crashes partway
through with `EPERM: Operation not permitted: failed to change lockfile
permissions (fchmod)` — the same class of DrvFs `chmod`/`fchmod`
limitation as the npm `--no-bin-links` issue below, just hitting Bun's
lockfile write instead of npm's bin-linking step. Workaround, verified
live:

```bash
# once, or whenever bun.lock needs regenerating — on native WSL filesystem,
# where fchmod works:
mkdir -p ~/bun-lockgen-tmp && cp package.json ~/bun-lockgen-tmp/
cd ~/bun-lockgen-tmp && bun install   # writes a clean bun.lock here
cp bun.lock /mnt/c/GitHub/my-idea/bun.lock

# back on DrvFs — --frozen-lockfile never rewrites the lockfile, so it
# doesn't hit the fchmod path and installs node_modules cleanly:
cd /mnt/c/GitHub/my-idea && bun install --frozen-lockfile
```

This is also why CI/release should always use `--frozen-lockfile` rather
than a bare `bun install` — it sidesteps this class of issue entirely,
not just as a DrvFs workaround.

## Agent rule: don't patch the base system

Before installing anything to fix a missing dependency, check whether it
belongs in `manifest.scm` instead:

```
# Bad
sudo apt install openjdk
npm install -g shadow-cljs

# Preferred
# add the package to manifest.scm, then:
guix shell -m manifest.scm -- bun run check
```

A missing dependency should normally become a `manifest.scm` change, not
an undocumented machine-local install. The legacy note below about npm's
`--no-bin-links` is kept for historical context (some scripts still
invoke `node_modules/<pkg>` binaries directly); day-to-day dependency
installs go through Bun now, per above.

`npm install` on this filesystem mount needed `--no-bin-links` (DrvFs
doesn't support the `chmod` npm's bin-linking step needs) — invoke
installed CLI tools directly via `node node_modules/<pkg>/<entry>.js`
rather than `npx`/`bunx` when bin-links are absent.

## If `cargo check` fails with "rustc X is not supported"

Tauri's dependency tree moves faster than Guix's packaged `rust`; a
transitive dependency can require a newer rustc than the channel commit
your `guix pull` last landed on provides. Two independent fixes, not
mutually exclusive:

- `guix time-machine -C <path-to-channels.scm> -- shell -m manifest.scm --
  <command>` — `my-lisp` maintains an ecosystem-wide `channels.scm` (its
  repo root) pinned to a Guix revision verified to build all four repos.
  Point at it directly rather than running your own `guix pull` first; a
  per-user `guix pull` only updates *that* user's channel state, not the
  shared profile or other users.
- `cargo update -p <crate> --precise <version>` as a narrower, temporary
  pin if you don't want to move the whole toolchain.

Frontend work (`shadow-cljs compile app`) doesn't depend on the Rust
toolchain version and can be verified independently of this.

## Reproducibility / evidence

A result is authoritative when it passes inside the declared Guix
environment (`guix shell --pure -m manifest.scm -- <command>`), not just
on whatever happens to be installed locally. When recording evidence (see
`evidence/README.md` in the neighboring repos), include the repo commit,
the command run, and expected vs. actual — the same fields those files
already require.

## How to check neighboring repositories

Read `my-lisp/ecosystem-status.my`, `fpga-lisp/ecosystem-status.md`,
`cml/ecosystem-status.md`/`compatibility.my`, and each neighbor's own
`evidence/` directory directly rather than asking another agent.

## Coordination channels

- Durable facts: `NOTE-*.md` files in each repo root, or `evidence/*.my`
  for fixture-level claims — not prose status messages.
- Cross-session messages: `send_message`/`list_sessions` (Claude Code
  sessions in this environment).
- Live semantics oracle: `my-lisp --tcp=9999 --protocol=sexpr` —
  `eval`/`parse`/`diagnose`/`contract-version`, one isolated environment
  per connection. Also carries `notify`/`poll` for short-lived pings
  (in-memory only, wiped on server restart, 500-entry cap — not a
  substitute for the durable channels above).
- OpenCode agent (different tool, no direct message channel) coordinates
  via `NOTE-*.md` and maintains a live snapshot at
  `C:\Users\user\Documents\GitHub\docs\AGENT_MEMORY.md` (not a git repo).
