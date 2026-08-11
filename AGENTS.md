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

## Agent rule: don't patch the base system

Before installing anything to fix a missing dependency, check whether it
belongs in `manifest.scm` instead:

```
# Bad
sudo apt install openjdk
npm install -g shadow-cljs

# Preferred
# add the package to manifest.scm, then:
guix shell -m manifest.scm -- npx shadow-cljs compile app
```

A missing dependency should normally become a `manifest.scm` change, not
an undocumented machine-local install. `npm install` on this filesystem
mount needs `--no-bin-links` (DrvFs doesn't support the `chmod` npm's
bin-linking step needs) — invoke installed CLI tools directly via `node
node_modules/<pkg>/<entry>.js` rather than `npx` when bin-links are absent.

## Known blocker: rustc version

Guix's `rust` package is 1.85.1; several transitive Tauri dependencies
(`darling`, `icu_*`, `time`, `zbus`, `plist`) need 1.86–1.88. `cargo check
--workspace` fails until either the Guix channel is updated (`guix pull`,
in progress ecosystem-wide as of 2026-08-12 — check the TCP mailbox or
`docs/AGENT_MEMORY.md` for status before re-attempting) or those crates are
pinned back with `cargo update --precise`. Frontend work (`shadow-cljs
compile app`) is unaffected and can be verified independently — it doesn't
depend on the Rust toolchain version.

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
