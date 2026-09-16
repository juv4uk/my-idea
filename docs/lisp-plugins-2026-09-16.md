# my-lisp plugins: our own Emacs

(Secondary/English mirror — see `lisp-plugins-2026-09-16.uk.md` for the
primary version.)

my-idea can be extended and configured by writing real my-lisp, the same
way Emacs is extended in Emacs Lisp rather than C. The host (Rust/Tauri)
supplies mechanism only: command registration, key bindings, event
subscriptions, buffer/selection snapshots, and applying an effect back.
What a command or handler actually does is entirely Lisp-owned.

The mechanism itself (issues #7/#9/#10/#11) already existed in the
codebase, fully tested in isolation — but `EditorCommandRegistry` was
never installed into the live session and no UI ever called it. This
change is what makes it actually reachable.

## Where plugins live

- `~/.config/my-idea/init.lisp` — loaded first, if present.
- `~/.config/my-idea/plugins/*.lisp` — loaded in filename order, each
  isolated (one broken plugin never blocks the rest).

Reload is explicit — the "⟲" button in my-idea's header. No file-watcher.

## Editor API

- `(editor/register-command "name" (lambda () ...) "description")`
- `(editor/keymap "Ctrl-Shift-b" "command-name")` — CodeMirror's own key
  string format; the host never interprets it.
- `(editor/on "event" "handler-id" (lambda () ...))` — currently fired
  events: `"after-open"`, `"before-save"`, `"after-save"`.
- `(editor/buffer-text)` / `(editor/selection)`
- `(editor/replace-selection "text")`
- `(editor/message "text")` — surfaces in the REPL console.

## Example

`docs/examples/plugins/hello.lisp`, kept honest by
`src-tauri/tests/example_plugin_contract.rs`.

## Command palette

The "⌘ Commands" header button (in the spirit of Emacs' `M-x`) lists and
invokes registered commands by name.

## Known limitation

A keymap added after a reload only takes effect on the next editor
(re)mount (tab switch / reopen) — CodeMirror's keymap isn't reactive.
Commands and `editor/on` hooks apply immediately.
