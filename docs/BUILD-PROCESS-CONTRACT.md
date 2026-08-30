# Build process contract

Status: implemented substrate; compiler adapters and UI wiring are separate
tasks.

## Boundary

`ProcessService` accepts an already selected, fixed command profile from a
trusted adapter. It is not exposed as a generic Tauri command and does not
accept a shell command string from the frontend.

Each `ProcessSpec` contains three separate fields:

- profile identity;
- executable path/name;
- argv vector.

The child always runs with the canonical open workspace as `cwd`, null stdin,
and piped stdout/stderr. On Unix it receives a new process group. Windows
cancellation uses `taskkill /T /F`, so descendants are included.

## Lifecycle

For each canonical workspace:

```text
idle -> running -> succeeded | failed | cancelled -> idle
```

A second start while that workspace is running is rejected before spawning.
Different workspaces remain independent. Spawn failure does not create an
active run.

Cancellation is a bounded control request to the platform process-tree
mechanism. `IDEA-STOP-BUILD` must additionally prove that a real compiler or
dev-server descendant does not survive it.

## Event schema 1

Every event contains:

```text
schema
runId
sequence
timestampMs
profile
stream = system | stdout | stderr
line
state = running | succeeded | failed | cancelled
exitCode = integer | null
```

Sequence numbers are allocated atomically across both output streams. Reader
threads are joined before the terminal event, so no stdout/stderr line from a
run can be emitted after its terminal state.

The terminal event is evidence about the child process only. It is not by
itself evidence that a requested compiler artifact exists; adapters must add
their own artifact verification.
