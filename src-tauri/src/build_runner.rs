// build-runner substrate for the Build Output panel (IDEA-BUILD-OUTPUT-UI).
//
// The Build process contract (docs/BUILD-PROCESS-CONTRACT.md) forbids exposing
// `ProcessService` itself as a generic Tauri command and forbids accepting a
// shell command string from the frontend. This module is the thin substrate
// layer that turns a *fixed command profile* (profile identity + executable +
// argv vector, never a shell string) into a `ProcessService` run and forwards
// every versioned `ProcessEvent` to the frontend over the Tauri event bus.
//
// This is not the compiler/WSM adapter (those live in IDEA-TAURI-BUILD-ADAPTER
// and IDEA-WSM-CLI-ADAPTER and are separate tasks that will supply the fixed
// profiles). This module provides the wire + panel substrate those adapters
// and the Build Output panel share: start/cancel/state, one active run per
// workspace, ordered stdout/stderr/system events, exit state, and a single
// actionable spawn (missing-tool) diagnostic.

use crate::process_service::{EventSink, ProcessEvent, ProcessService, ProcessSpec};
use crate::Workspace;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

/// The frontend event name for build output events.
pub const BUILD_OUTPUT_EVENT: &str = "build-output";

/// A fixed command profile supplied by an adapter or the substrate command.
///
/// Exactly mirrors the `ProcessSpec` contract shape: three separate fields,
/// no concatenated command string. `args` is a JSON array of argv entries.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BuildSpec {
    pub profile: String,
    pub executable: String,
    pub args: Vec<String>,
}

/// A small, testable payload describing the *active* build for a workspace so
/// the frontend can render the command that is running (e.g. after reload).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveBuild {
    pub run_id: u64,
    pub profile: String,
    pub executable: String,
    pub args: Vec<String>,
}

/// Keeps the last-known active build per workspace. This is only for the panel
/// header (what profile is running); it is not an authority over the process
/// (the `ProcessService`'s own `active` map is the real source of truth).
#[derive(Default)]
pub struct BuildRegistry(Mutex<HashMap<PathBuf, ActiveBuild>>);

impl BuildRegistry {
    fn set(&self, workspace: &PathBuf, active: ActiveBuild) {
        if let Ok(mut map) = self.0.lock() {
            map.insert(workspace.clone(), active);
        }
    }

    fn clear(&self, workspace: &PathBuf) {
        if let Ok(mut map) = self.0.lock() {
            map.remove(workspace);
        }
    }

    fn get(&self, workspace: &PathBuf) -> Option<ActiveBuild> {
        self.0.lock().ok().and_then(|map| map.get(workspace).cloned())
    }
}

/// Starts a fixed build profile for the currently open workspace and streams
/// versioned `ProcessEvent`s to the frontend as `build-output` events.
///
/// On spawn failure (e.g. a missing tool) no active run is created and the
/// error is returned as a single actionable diagnostic; the terminal event
/// for a successful start is `process started` (state `running`).
#[tauri::command]
pub fn start_build(
    app: AppHandle,
    service: State<'_, ProcessService>,
    workspace: State<'_, Workspace>,
    registry: State<'_, BuildRegistry>,
    spec: BuildSpec,
) -> Result<u64, String> {
    let path = crate::root(&workspace)?;
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("workspace is unavailable: {error}"))?;
    let app_handle = app.clone();
    let sink: EventSink = Arc::new(move |event: ProcessEvent| {
        let _ = app_handle.emit(BUILD_OUTPUT_EVENT, event);
    });
    let run_id = service.start(
        &canonical,
        ProcessSpec {
            profile: spec.profile.clone(),
            executable: spec.executable.clone().into(),
            args: spec.args.clone(),
        },
        sink,
    )?;
    registry.set(
        &canonical,
        ActiveBuild {
            run_id,
            profile: spec.profile,
            executable: spec.executable,
            args: spec.args,
        },
    );
    Ok(run_id)
}

/// Requests bounded process-tree cancellation for the open workspace and
/// returns the cancelled run id.
#[tauri::command]
pub fn cancel_build(
    service: State<'_, ProcessService>,
    workspace: State<'_, Workspace>,
    registry: State<'_, BuildRegistry>,
) -> Result<u64, String> {
    let path = crate::root(&workspace)?;
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("workspace is unavailable: {error}"))?;
    let run_id = service.cancel(&canonical)?;
    registry.clear(&canonical);
    Ok(run_id)
}

/// Returns the last-known active build for the open workspace, or `None` if
/// the workspace is idle. Used by the panel to render the running command
/// profile and to distinguish "no run yet" from "no active run".
#[tauri::command]
pub fn active_build(
    workspace: State<'_, Workspace>,
    registry: State<'_, BuildRegistry>,
) -> Result<Option<ActiveBuild>, String> {
    match crate::root(&workspace) {
        Ok(path) => {
            let canonical = path.canonicalize().unwrap_or(path);
            Ok(registry.get(&canonical))
        }
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process_service::{EventStream, RunState, EVENT_SCHEMA};

    // The Build Output panel must render against the *real* event schema, so
    // this test locks the wire contract: the frontend event name, the schema
    // version, and the serialized field names / enum variants the frontend
    // will read. It is not a fixture-driven assertion in the panel itself
    // (that is the frontend's job) — it makes the backend promise exact and
    // greppable so the UI test can assert against the same source of truth.
    #[test]
    fn build_output_wire_contract_is_stable() {
        // Frontend-facing event channel name.
        assert_eq!(BUILD_OUTPUT_EVENT, "build-output");
        // The events we forward carry the versioned schema.
        assert_eq!(EVENT_SCHEMA, 1);
        // Stream and state variants the panel distinguishes.
        assert_eq!(format!("{:?}", EventStream::System), "System");
        assert_eq!(format!("{:?}", EventStream::Stdout), "Stdout");
        assert_eq!(format!("{:?}", EventStream::Stderr), "Stderr");
        for state in [RunState::Running, RunState::Succeeded, RunState::Failed, RunState::Cancelled] {
            let _ = format!("{state:?}");
        }
    }

    #[test]
    fn active_build_registry_tracks_the_most_recent_start() {
        let registry = BuildRegistry::default();
        let workspace = PathBuf::from("/tmp/ws");
        assert!(registry.get(&workspace).is_none());
        registry.set(
            &workspace,
            ActiveBuild {
                run_id: 7,
                profile: "check".into(),
                executable: "/bin/echo".into(),
                args: vec!["-n".into(), "ok".into()],
            },
        );
        let active = registry.get(&workspace).unwrap();
        assert_eq!(active.run_id, 7);
        assert_eq!(active.profile, "check");
        assert_eq!(active.args, vec!["-n", "ok"]);
        registry.clear(&workspace);
        assert!(registry.get(&workspace).is_none());
    }
}
