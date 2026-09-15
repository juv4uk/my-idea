# Lisp-extensible, self-building my-idea

Date: 2026-09-15
Status: design direction approved by project owner

## Goal

`my-idea` should become a small Emacs-like environment in one precise sense: the native shell supplies mechanisms, while user-visible editor behaviour can increasingly be defined by `my-lisp` plugins.

The same IDE should also be able to initiate a build of its own next version. That build is explicitly a staged self-build using the project compiler/toolchain plus Tauri; it is not called full self-hosting until the bootstrap dependencies have actually been removed.

## Authority boundary

- Tauri/Rust/ClojureScript own platform and UI mechanisms.
- `my-lisp` owns plugin programs and their language semantics.
- Editor APIs expose capabilities to Lisp; they do not duplicate Lisp semantics in the UI.
- CML/compiler may lower project-owned source where supported; Tauri remains the platform build/bundle mechanism.

## Minimal architecture

```text
my-idea native shell
  |-- persistent my-lisp session / REPL
  |-- Editor API
  |     |-- command
  |     |-- buffer-text / selection / replace-selection
  |     |-- message
  |     |-- on (hooks/events)
  |     `-- keymap
  |-- plugin loader
  |     |-- ~/.config/my-idea/init.lisp
  |     `-- ~/.config/my-idea/plugins/*.lisp
  `-- self-build orchestrator
        |-- project compiler / CML stage
        `-- Tauri build + bundle stage
```

## Delivery slices

1. Persistent REPL + console launch (`my-idea [path]`).
2. Minimal Editor API, with one executable Lisp plugin proving command registration.
3. Plugin discovery/loading plus explicit reload and isolated diagnostics.
4. Hooks/keymaps so useful editor behaviour can live in Lisp.
5. Self-build command that emits an inspectable build plan before execution.
6. Compiler/CML pre-build stage connected to Tauri's build pipeline.
7. Generation witness: `my-idea_0 -> my-idea_1 -> my-idea_2`, with `my-idea_1` able to initiate the build of `my-idea_2`.

## Non-goals for the first slices

- no arbitrary UI widget construction from Lisp yet;
- no PTY/terminal framework unless transparent stdio proves insufficient;
- no claim of full self-hosting;
- no second evaluator in ClojureScript or Rust;
- no plugin package manager before local plugins work cleanly.

## Executable evidence

Each slice starts RED. The key witnesses are: persistent definitions across REPL evaluations; a `.lisp` plugin registering and invoking a real editor command; plugin failure not crashing the editor; deterministic self-build plan; Tauri build consuming the compiler-produced stage; and a two-generation self-build witness.
