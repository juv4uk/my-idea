# Implementation plan: Lisp-extensible self-building my-idea

This plan intentionally preserves small independently provable slices.

## 1. REPL + CLI foundation
- RED: one session retains definitions across sequential evaluations.
- RED: zero/one startup path parsing.
- GREEN: persistent desktop session and `my-idea [path]` launch.
- Verify lifecycle cleanup and existing gates.

## 2. Editor API seed
- RED: a Lisp plugin cannot yet register a command through an explicit Editor API.
- Add only the mechanism needed for `command`, `message`, selection/buffer access and replacement.
- Prove one plugin-defined command end-to-end without Lisp expected answers in UI code.

## 3. Plugin loader
- RED: deterministic discovery order and isolated failure diagnostics.
- Load `init.lisp` and `plugins/*.lisp` into the persistent session.
- Add explicit reload; do not add a package manager.

## 4. Hooks and keymaps
- RED: Lisp-defined hook/key binding is observable through editor behaviour.
- Add `on` and `keymap` mechanisms with bounded capability exposure.

## 5. Inspectable self-build
- RED: self-build plan is deterministic and names every external stage.
- Add a build-plan command before any execution command.
- Separate compiler/CML stage from Tauri build/bundle stage.

## 6. Tauri integration
- RED: build fails if the compiler stage is omitted/stale.
- Connect project compiler output to the Tauri pre-build path.
- Keep Tauri as an explicit bootstrap/platform dependency.

## 7. Generation witness
- Build generation 1 from generation 0.
- Use generation 1 to initiate generation 2.
- Compare declared inputs/toolchain and verify generation 2 passes normal smoke/build gates.
- Only strengthen self-hosting claims to the level actually demonstrated.
