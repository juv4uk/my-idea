use serde::Serialize;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

pub const EVENT_SCHEMA: u16 = 1;

#[derive(Clone, Debug)]
pub struct ProcessSpec {
    pub profile: String,
    pub executable: PathBuf,
    pub args: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RunState {
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EventStream {
    System,
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessEvent {
    pub schema: u16,
    pub run_id: u64,
    pub sequence: u64,
    pub timestamp_ms: u128,
    pub profile: String,
    pub stream: EventStream,
    pub line: String,
    pub state: RunState,
    pub exit_code: Option<i32>,
}

pub type EventSink = Arc<dyn Fn(ProcessEvent) + Send + Sync + 'static>;

#[derive(Clone, Copy)]
struct ActiveRun {
    run_id: u64,
    process_id: u32,
}

#[derive(Clone, Default)]
pub struct ProcessService {
    next_run_id: Arc<AtomicU64>,
    active: Arc<Mutex<HashMap<PathBuf, ActiveRun>>>,
}

impl ProcessService {
    pub fn start(
        &self,
        workspace: &Path,
        spec: ProcessSpec,
        sink: EventSink,
    ) -> Result<u64, String> {
        let workspace = workspace
            .canonicalize()
            .map_err(|error| format!("workspace is unavailable: {error}"))?;
        if !workspace.is_dir() {
            return Err("workspace is not a directory".into());
        }
        if spec.executable.as_os_str().is_empty() {
            return Err("build profile has no executable".into());
        }

        let mut command = Command::new(&spec.executable);
        command
            .args(&spec.args)
            .current_dir(&workspace)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }

        let run_id = self.next_run_id.fetch_add(1, Ordering::Relaxed) + 1;
        let mut active = self
            .active
            .lock()
            .map_err(|_| "process service lock is poisoned".to_string())?;
        if active.contains_key(&workspace) {
            return Err("a build is already running for this workspace".into());
        }

        let mut child = command.spawn().map_err(|error| {
            format!("could not start build profile '{}': {error}", spec.profile)
        })?;
        let process_id = child.id();
        active.insert(workspace.clone(), ActiveRun { run_id, process_id });
        drop(active);

        let sequence = Arc::new(AtomicU64::new(0));
        emit(
            &sink,
            &sequence,
            run_id,
            &spec.profile,
            EventStream::System,
            "process started".into(),
            RunState::Running,
            None,
        );
        let stdout_thread = child.stdout.take().map(|stdout| {
            forward_lines(
                stdout,
                EventStream::Stdout,
                run_id,
                spec.profile.clone(),
                sequence.clone(),
                sink.clone(),
            )
        });
        let stderr_thread = child.stderr.take().map(|stderr| {
            forward_lines(
                stderr,
                EventStream::Stderr,
                run_id,
                spec.profile.clone(),
                sequence.clone(),
                sink.clone(),
            )
        });

        let active_runs = self.active.clone();
        thread::spawn(move || {
            let status = child.wait();
            if let Some(handle) = stdout_thread {
                let _ = handle.join();
            }
            if let Some(handle) = stderr_thread {
                let _ = handle.join();
            }
            let cancelled = active_runs
                .lock()
                .ok()
                .and_then(|mut runs| runs.remove(&workspace))
                .is_none();
            let (state, exit_code, line) = match status {
                Ok(status) if cancelled => {
                    (RunState::Cancelled, status.code(), "process cancelled")
                }
                Ok(status) if status.success() => {
                    (RunState::Succeeded, status.code(), "process succeeded")
                }
                Ok(status) => (RunState::Failed, status.code(), "process failed"),
                Err(_) => (RunState::Failed, None, "could not wait for process"),
            };
            emit(
                &sink,
                &sequence,
                run_id,
                &spec.profile,
                EventStream::System,
                line.into(),
                state,
                exit_code,
            );
        });
        Ok(run_id)
    }

    pub fn cancel(&self, workspace: &Path) -> Result<u64, String> {
        let workspace = workspace
            .canonicalize()
            .map_err(|error| format!("workspace is unavailable: {error}"))?;
        let mut active_runs = self
            .active
            .lock()
            .map_err(|_| "process service lock is poisoned".to_string())?;
        let active = active_runs
            .get(&workspace)
            .copied()
            .ok_or_else(|| "no build is running for this workspace".to_string())?;
        terminate_process_tree(active.process_id)?;
        active_runs.remove(&workspace);
        Ok(active.run_id)
    }
}

fn forward_lines(
    reader: impl std::io::Read + Send + 'static,
    stream: EventStream,
    run_id: u64,
    profile: String,
    sequence: Arc<AtomicU64>,
    sink: EventSink,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            emit(
                &sink,
                &sequence,
                run_id,
                &profile,
                stream,
                line,
                RunState::Running,
                None,
            );
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn emit(
    sink: &EventSink,
    sequence: &AtomicU64,
    run_id: u64,
    profile: &str,
    stream: EventStream,
    line: String,
    state: RunState,
    exit_code: Option<i32>,
) {
    sink(ProcessEvent {
        schema: EVENT_SCHEMA,
        run_id,
        sequence: sequence.fetch_add(1, Ordering::Relaxed),
        timestamp_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        profile: profile.into(),
        stream,
        line,
        state,
        exit_code,
    });
}

#[cfg(unix)]
fn terminate_process_tree(process_id: u32) -> Result<(), String> {
    let process_group = format!("-{process_id}");
    let status = Command::new("kill")
        .args(["-TERM", "--", &process_group])
        .status()
        .map_err(|error| format!("could not request process-group cancellation: {error}"))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| "process-group cancellation failed".into())
}

#[cfg(windows)]
fn terminate_process_tree(process_id: u32) -> Result<(), String> {
    let process_id = process_id.to_string();
    let status = Command::new("taskkill")
        .args(["/PID", &process_id, "/T", "/F"])
        .status()
        .map_err(|error| format!("could not request Windows process-tree cancellation: {error}"))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| "Windows process-tree cancellation failed".into())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::mpsc,
        time::{Duration, SystemTime},
    };

    fn workspace(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("my-idea-process-{label}-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path.canonicalize().unwrap()
    }

    #[test]
    fn streams_versioned_output_and_success_state() {
        let workspace = workspace("output");
        let service = ProcessService::default();
        let (sender, receiver) = mpsc::channel();
        let run_id = service
            .start(
                &workspace,
                ProcessSpec {
                    profile: "test-echo".into(),
                    executable: "/bin/echo".into(),
                    args: vec!["hello world".into()],
                },
                Arc::new(move |event| sender.send(event).unwrap()),
            )
            .unwrap();

        let mut events = Vec::new();
        loop {
            let event = receiver.recv_timeout(Duration::from_secs(2)).unwrap();
            let terminal = event.state != RunState::Running;
            events.push(event);
            if terminal {
                break;
            }
        }
        assert!(events.iter().all(|event| event.schema == EVENT_SCHEMA));
        assert!(events.iter().all(|event| event.run_id == run_id));
        assert!(events.iter().any(|event| event.line == "hello world"));
        assert_eq!(events.last().unwrap().state, RunState::Succeeded);
        assert!(events
            .windows(2)
            .all(|pair| pair[0].sequence < pair[1].sequence));
        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn refuses_a_second_run_and_cancels_the_process_group() {
        let workspace = workspace("single");
        let service = ProcessService::default();
        let sink: EventSink = Arc::new(|_| {});
        let first = service
            .start(
                &workspace,
                ProcessSpec {
                    profile: "test-sleep".into(),
                    executable: "/bin/sleep".into(),
                    args: vec!["30".into()],
                },
                sink.clone(),
            )
            .unwrap();
        let second = service.start(
            &workspace,
            ProcessSpec {
                profile: "test-second".into(),
                executable: "/bin/echo".into(),
                args: vec!["must-not-run".into()],
            },
            sink,
        );

        assert_eq!(
            second.unwrap_err(),
            "a build is already running for this workspace"
        );
        assert_eq!(service.cancel(&workspace).unwrap(), first);
        thread::sleep(Duration::from_millis(50));
        fs::remove_dir_all(workspace).unwrap();
    }
}
