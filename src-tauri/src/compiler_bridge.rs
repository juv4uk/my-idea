use std::{
    fmt,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticStream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerDiagnostic {
    stream: DiagnosticStream,
    path: String,
    line: Option<usize>,
    column: Option<usize>,
    message: String,
}

impl CompilerDiagnostic {
    pub fn new(
        stream: DiagnosticStream,
        path: impl Into<String>,
        line: Option<usize>,
        column: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            stream,
            path: path.into(),
            line,
            column,
            message: message.into(),
        }
    }

    pub fn stream(&self) -> DiagnosticStream {
        self.stream
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn line(&self) -> Option<usize> {
        self.line
    }

    pub fn column(&self) -> Option<usize> {
        self.column
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerRequest {
    source: PathBuf,
    target: String,
}

impl CompilerRequest {
    pub fn new(source: impl Into<PathBuf>, target: impl Into<String>) -> Result<Self, String> {
        let source = source.into();
        let target = target.into();
        if source.extension().and_then(|value| value.to_str()) != Some("lisp") {
            return Err("compiler source must use the canonical .lisp extension".into());
        }
        if target.trim().is_empty() {
            return Err("compiler target must not be empty".into());
        }
        Ok(Self { source, target })
    }

    pub fn source(&self) -> PathBuf {
        self.source.clone()
    }

    pub fn target(&self) -> &str {
        &self.target
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerArtifact {
    path: PathBuf,
    compiler: String,
    compiler_revision: String,
    input_revision: String,
}

impl CompilerArtifact {
    pub fn new(
        path: impl Into<PathBuf>,
        compiler: impl Into<String>,
        compiler_revision: impl Into<String>,
        input_revision: impl Into<String>,
    ) -> Result<Self, String> {
        let path = path.into();
        let compiler = compiler.into();
        let compiler_revision = compiler_revision.into();
        let input_revision = input_revision.into();
        if path.as_os_str().is_empty() {
            return Err("compiler artifact path must not be empty".into());
        }
        if compiler.trim().is_empty()
            || compiler_revision.trim().is_empty()
            || input_revision.trim().is_empty()
        {
            return Err("compiler artifact provenance must be explicit".into());
        }
        Ok(Self {
            path,
            compiler,
            compiler_revision,
            input_revision,
        })
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn compiler(&self) -> &str {
        &self.compiler
    }

    pub fn compiler_revision(&self) -> &str {
        &self.compiler_revision
    }

    pub fn input_revision(&self) -> &str {
        &self.input_revision
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerRun {
    artifact: CompilerArtifact,
    diagnostics: Vec<CompilerDiagnostic>,
}

impl CompilerRun {
    pub fn artifact(&self) -> &CompilerArtifact {
        &self.artifact
    }

    pub fn diagnostics(&self) -> &[CompilerDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerFailure {
    message: String,
    diagnostics: Vec<CompilerDiagnostic>,
    exit_code: Option<i32>,
}

impl CompilerFailure {
    fn new(
        message: impl Into<String>,
        diagnostics: Vec<CompilerDiagnostic>,
        exit_code: Option<i32>,
    ) -> Self {
        Self {
            message: message.into(),
            diagnostics,
            exit_code,
        }
    }

    pub fn diagnostics(&self) -> &[CompilerDiagnostic] {
        &self.diagnostics
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }
}

impl fmt::Display for CompilerFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CompilerFailure {}

#[derive(Clone, Debug)]
pub struct CompilerBridge {
    executable: PathBuf,
}

impl CompilerBridge {
    pub fn at(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
        }
    }

    pub fn compile(&self, request: &CompilerRequest) -> Result<CompilerArtifact, CompilerFailure> {
        self.compile_observed(request).map(|run| run.artifact)
    }

    pub fn compile_observed(&self, request: &CompilerRequest) -> Result<CompilerRun, CompilerFailure> {
        if !self.executable.is_file() {
            return Err(CompilerFailure::new(
                format!(
                    "authoritative compiler is unavailable: {}",
                    self.executable.display()
                ),
                Vec::new(),
                None,
            ));
        }
        if !request.source.is_file() {
            return Err(CompilerFailure::new(
                format!("compiler source is unavailable: {}", request.source.display()),
                Vec::new(),
                None,
            ));
        }
        if request.target != "x86_64-linux" {
            return Err(CompilerFailure::new(
                format!("compiler target is unsupported: {}", request.target),
                Vec::new(),
                None,
            ));
        }

        let source_modified = modified(&request.source)?;
        let output_path = artifact_path(request)?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                CompilerFailure::new(
                    format!("compiler artifact directory could not be created: {error}"),
                    Vec::new(),
                    None,
                )
            })?;
        }
        if output_path.exists() {
            fs::remove_file(&output_path).map_err(|error| {
                CompilerFailure::new(
                    format!("stale compiler artifact could not be removed: {error}"),
                    Vec::new(),
                    None,
                )
            })?;
        }

        let output = Command::new(&self.executable)
            .arg("x86-elf")
            .arg(&request.source)
            .arg(&output_path)
            .output()
            .map_err(|error| {
                CompilerFailure::new(
                    format!("authoritative compiler could not be started: {error}"),
                    Vec::new(),
                    None,
                )
            })?;

        let diagnostics = diagnostics_from_output(request, &output.stdout, &output.stderr);
        if !output.status.success() {
            return Err(CompilerFailure::new(
                "authoritative compiler returned a non-zero exit status",
                diagnostics,
                output.status.code(),
            ));
        }
        if !output_path.is_file() {
            return Err(CompilerFailure::new(
                "authoritative compiler reported success without producing an artifact",
                diagnostics,
                output.status.code(),
            ));
        }
        let artifact_modified = modified(&output_path)?;
        if artifact_modified < source_modified {
            return Err(CompilerFailure::new(
                "authoritative compiler produced a stale artifact",
                diagnostics,
                output.status.code(),
            ));
        }

        let artifact = CompilerArtifact::new(
            &output_path,
            compiler_identity(&self.executable),
            git_revision_for(&self.executable),
            git_revision_for(&request.source),
        )
        .map_err(|message| {
            CompilerFailure::new(message, diagnostics.clone(), output.status.code())
        })?;

        Ok(CompilerRun {
            artifact,
            diagnostics,
        })
    }
}

fn artifact_path(request: &CompilerRequest) -> Result<PathBuf, CompilerFailure> {
    let stem = request
        .source
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            CompilerFailure::new("compiler source has no valid file stem", Vec::new(), None)
        })?;
    Ok(PathBuf::from("target")
        .join("my-idea")
        .join(format!("{stem}-{}", request.target)))
}

fn modified(path: &Path) -> Result<SystemTime, CompilerFailure> {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .map_err(|error| {
            CompilerFailure::new(
                format!("compiler provenance metadata is unavailable: {error}"),
                Vec::new(),
                None,
            )
        })
}

fn diagnostics_from_output(
    request: &CompilerRequest,
    stdout: &[u8],
    stderr: &[u8],
) -> Vec<CompilerDiagnostic> {
    let path = request.source.to_string_lossy().into_owned();
    let mut diagnostics = Vec::new();
    diagnostics.extend(lines(stdout).into_iter().map(|message| {
        CompilerDiagnostic::new(DiagnosticStream::Stdout, path.clone(), None, None, message)
    }));
    diagnostics.extend(lines(stderr).into_iter().map(|message| {
        CompilerDiagnostic::new(DiagnosticStream::Stderr, path.clone(), None, None, message)
    }));
    diagnostics
}

fn lines(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

pub(crate) fn compiler_identity(executable: &Path) -> String {
    executable
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("compiler")
        .to_owned()
}

pub(crate) fn git_revision_for(path: &Path) -> String {
    let start = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };
    let Some(root) = start
        .ancestors()
        .find(|ancestor| ancestor.join(".git").exists())
    else {
        return "unavailable".into();
    };
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|revision| revision.trim().to_owned())
        .filter(|revision| !revision.is_empty())
        .unwrap_or_else(|| "unavailable".into())
}
