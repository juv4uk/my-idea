# Who wins a key: an explicit precedence model

(Secondary/English mirror — see `keymap-authority-2026-09-16.uk.md` for
the primary version.)

my-idea now resolves key conflicts explicitly and transparently rather
than silently — the resolution is queryable (`resolve_keymaps`),
deterministic for a given set of bindings, and no participant wins just
by sitting first in CodeMirror's extension array.

## Where this came from

F12 was simultaneously CodeMirror's own go-to-definition binding and the
WebView's standard DevTools shortcut once the `devtools` feature shipped
— nothing in the code could honestly say which one would actually fire.

## The concrete F12 fix

- **F12 stays DevTools' alone** (host/WebView level, outside our control).
- **Go-to-definition moved to `Alt-.`** — Emacs' own `M-.`
  (`xref-find-definitions`), matching the plugin system's Emacs framing.

## Precedence model (`keymap_authority.rs`)

1. **UserPlugin** (`editor/keymap` from a `.lisp` plugin) — highest.
   Matches Emacs: your init.el config overrides even Emacs' own defaults.
2. **BuiltIn** (our own commands, e.g. go-to-definition).
3. **Host-reserved** (currently just `F12`) — not a priority level, a
   veto: no participant ever gets it; the attempt stays visible as a
   `candidate`, `winner` is always `null`.

CodeMirror's own defaults are never enumerated — they already sit at the
lowest priority by construction, and this deliberately avoids the
ever-growing "list every CodeMirror key" table issue #51 warns against.

## Technical flow

```text
resolve-keymaps! (CLJS)
  → resolve_keymaps (Rust: real editor/keymap bindings + CLJS-reported built-ins)
  → keymap_authority::resolve (pure, deterministic)
  → only user-plugin winners become live CodeMirror keymaps
  → dispatch-editor-key! stays the fail-closed authority regardless
