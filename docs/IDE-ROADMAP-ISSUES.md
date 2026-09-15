# Issue-ready roadmap

The architectural design is split into independently mergeable evidence slices:

- IDE-REPL-1 — persistent REPL + console launch (issue #7).
- IDE-LISP-API-1 — minimal Editor API exposed to my-lisp; first plugin-defined command.
- IDE-LISP-PLUGINS-1 — deterministic `init.lisp` / `plugins/*.lisp` loading and reload.
- IDE-LISP-HOOKS-1 — hooks and keymaps defined in my-lisp.
- IDE-SELF-BUILD-1 — deterministic, inspectable self-build plan.
- IDE-SELF-BUILD-2 — compiler/CML stage feeding Tauri build/bundle.
- IDE-SELF-BUILD-3 — generation witness `my-idea_0 -> my-idea_1 -> my-idea_2`.
- IDE-DEPS-1 — advance `external/my-lisp` from its stale pin only after compatibility gates pass.
