# Vision: my-idea as the first face of the Lisp Machine

Status: long-range architectural vision. Do not widen current implementation slices merely to satisfy this document.

## Thesis

`my-idea` should evolve from "a Tauri application that evaluates my-lisp" toward **the first programmable face of our Lisp Machine**.

```text
                 my-idea
                    |
          UI / Editor capabilities
                    |
              Lisp Machine
              /         \
             /           \
          CML        Host capabilities
           |              |
          CPU           Tauri
                          |
                  Linux / Windows
```

Tauri is the current shell/body, not the owner of Lisp semantics.

## What we deliberately reuse

Tauri and its system WebView let us reuse mature platform machinery rather than create it prematurely: windows, rendering, input, filesystem/process/network integration and Web content rendering.

We should not build our own browser engine just to claim independence. If the IDE needs Web access, expose the smallest useful Web capability and let the host adapter use the available WebView.

## Power direction

Wrong long-term direction:

```text
Tauri app
  -> Rust/JS owns project meaning
  -> Lisp is one supported feature
```

Desired direction:

```text
Lisp Machine
  -> commands
  -> plugins
  -> hooks/keymaps
  -> project/build policy
  -> editor behavior
       |
       v
explicit capabilities
       |
       v
Tauri / WebView / OS mechanisms
```

## Replaceability test

Tauri is architecturally healthy only while it is replaceable.

A future second host, especially WSM OS Lisp, should be able to implement the same capability contracts without changing canonical Lisp semantics.

```text
                  Lisp Machine
                       |
                Capability ABI/API
                  /            \
                 /              \
          Tauri adapter      WSM OS adapter
              |                  |
       current desktop       future native
```

## Relationship to current roadmap

This vision comes after the existing evidence chain:

1. persistent REPL;
2. minimal Lisp Editor API;
3. `init.lisp` and plugins;
4. hooks/keymaps;
5. first-class compiler/CML bridge;
6. deterministic self-build provenance;
7. generation witness `my-idea0 -> my-idea1 -> my-idea2`.

Only then should host-independence become an implementation slice.

## Future executable evidence

RED first:

- one `.lisp` command behaves identically through a mock/reference host and Tauri host;
- capability absence fails closed;
- WebView receives navigation/render requests as mechanism data, without containing expected Lisp semantics;
- plugin code does not import Tauri-specific concepts;
- host and compiler provenance are inspectable;
- a WSM OS Lisp adapter can eventually satisfy the same contract.

## Rule for new features

Before adding a feature to Rust/CLJS, ask:

> Is this mechanism required to reach the host, or behavior that Lisp itself can own?

If it is behavior, prefer Lisp. If it is host mechanism, expose the narrowest explicit capability that is actually needed.

The goal is not a large IDE. The goal is **a small self-improving Lisp environment whose current body happens to be Tauri.**
