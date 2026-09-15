# Task acceptance contracts

## IDE-LISP-API-1
A plugin written in my-lisp registers and invokes one editor command through an explicit Editor API. Native code exposes mechanism only. RED first.

## IDE-LISP-PLUGINS-1
`init.lisp` and local `plugins/*.lisp` load deterministically into the persistent REPL session. One broken plugin yields a bounded diagnostic and does not crash the IDE. RED first.

## IDE-LISP-HOOKS-1
A Lisp plugin can bind one key and subscribe to one editor event without UI-owned Lisp semantics. RED first.

## IDE-SELF-BUILD-1
The IDE can produce a deterministic, inspectable plan for building its next generation, including compiler/CML and Tauri stages, without claiming full self-hosting. RED first.

## IDE-SELF-BUILD-2
A compiler/CML-produced project stage is a declared input to Tauri build. Missing/stale compiler output fails closed. Tauri remains explicit platform/bootstrap mechanism. RED first.

## IDE-SELF-BUILD-3
Generation 0 builds generation 1; generation 1 initiates generation 2; generation 2 passes the agreed smoke/build gates. The claim is limited to what this witness proves. RED first.

## IDE-DEPS-1
Update `external/my-lisp` from the recorded stale gitlink only in an isolated dependency change. The exact new SHA must be recorded and normal JS/Rust/Tauri gates must pass before merge.
