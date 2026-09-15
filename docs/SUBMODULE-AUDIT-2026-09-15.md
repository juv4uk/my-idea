# Submodule audit — 2026-09-15

`my-idea` currently declares one Git submodule:

- `external/my-lisp` -> `https://github.com/juv4uk/my-lisp.git`

Observed pins at audit time:

- `my-idea/main` submodule gitlink: `5500ac2ead9d028054d8a9452ecca82d1f678cc5`
- `my-lisp/main`: `e9485b5332fe0e2b562c9ed1cb8b70233b35b75c`

Result: **the submodule is not pinned to current `my-lisp/main`**.

Do not advance this gitlink blindly: `my-idea` builds against `my-lisp`, so the update must be a separate dependency change with the normal test/build gates. This audit records the mismatch so it cannot be mistaken for an up-to-date dependency.
