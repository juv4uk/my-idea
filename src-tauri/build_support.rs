use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    process::Command,
};

pub const SELF_BUILD_SOURCE_PATH: &str = "self-build/my-idea.lisp";
pub const SELF_BUILD_ARTIFACT_PATH: &str = "target/self-build/my-idea-x86_64-linux";
pub const SELF_BUILD_TARGET: &str = "x86_64-linux";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerStageGate {
    plan_digest: String,
    source_revision: String,
    source_path: String,
    source_sha256: String,
    compiler: String,
    compiler_revision: String,
    compiler_target: String,
    artifact_path: String,
    artifact_sha256: String,
}

impl CompilerStageGate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        plan_digest: impl Into<String>,
        source_revision: impl Into<String>,
        source_path: impl Into<String>,
        source_sha256: impl Into<String>,
        compiler: impl Into<String>,
        compiler_revision: impl Into<String>,
        compiler_target: impl Into<String>,
        artifact_path: impl Into<String>,
        artifact_sha256: impl Into<String>,
    ) -> Result<Self, String> {
        let gate = Self {
            plan_digest: plan_digest.into(),
            source_revision: source_revision.into(),
            source_path: source_path.into(),
            source_sha256: source_sha256.into(),
            compiler: compiler.into(),
            compiler_revision: compiler_revision.into(),
            compiler_target: compiler_target.into(),
            artifact_path: artifact_path.into(),
            artifact_sha256: artifact_sha256.into(),
        };
        gate.validate_shape()?;
        Ok(gate)
    }

    fn validate_shape(&self) -> Result<(), String> {
        require_hex("self-build plan digest", &self.plan_digest, 64)?;
        require_hex("source revision", &self.source_revision, 40)?;
        require_hex("source SHA-256", &self.source_sha256, 64)?;
        require_hex("compiler revision", &self.compiler_revision, 40)?;
        require_hex("artifact SHA-256", &self.artifact_sha256, 64)?;

        if self.source_path != SELF_BUILD_SOURCE_PATH {
            return Err(format!(
                "self-build source path must be {SELF_BUILD_SOURCE_PATH}"
            ));
        }
        if self.artifact_path != SELF_BUILD_ARTIFACT_PATH {
            return Err(format!(
                "self-build artifact path must be {SELF_BUILD_ARTIFACT_PATH}"
            ));
        }
        if self.compiler != "cml-compile" {
            return Err("self-build compiler must be cml-compile".into());
        }
        if self.compiler_target != SELF_BUILD_TARGET {
            return Err(format!(
                "self-build compiler target must be {SELF_BUILD_TARGET}"
            ));
        }
        Ok(())
    }

    pub fn plan_digest(&self) -> &str { &self.plan_digest }
    pub fn source_revision(&self) -> &str { &self.source_revision }
    pub fn source_path(&self) -> &str { &self.source_path }
    pub fn source_sha256(&self) -> &str { &self.source_sha256 }
    pub fn compiler(&self) -> &str { &self.compiler }
    pub fn compiler_revision(&self) -> &str { &self.compiler_revision }
    pub fn compiler_target(&self) -> &str { &self.compiler_target }
    pub fn artifact_path(&self) -> &str { &self.artifact_path }
    pub fn artifact_sha256(&self) -> &str { &self.artifact_sha256 }
}

fn require_hex(label: &str, value: &str, length: usize) -> Result<(), String> {
    if value.len() != length
        || !value
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
    {
        return Err(format!(
            "{label} must be lowercase {length}-hex provenance"
        ));
    }
    Ok(())
}

fn hash_file(path: &Path, label: &str) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("{label} is unavailable at {}: {error}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn git_output(repo_root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .map_err(|error| format!("git provenance probe could not start: {error}"))?;
    if !output.status.success() {
        return Err(format!("git provenance probe failed for args {args:?}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub fn verify_compiler_stage_gate(
    repo_root: &Path,
    gate: &CompilerStageGate,
) -> Result<(), String> {
    gate.validate_shape()?;

    let source = repo_root.join(gate.source_path());
    let source_sha = hash_file(&source, "self-build source")?;
    if source_sha != gate.source_sha256() {
        return Err(format!(
            "source SHA-256 mismatch: expected={} actual={source_sha}",
            gate.source_sha256()
        ));
    }

    let artifact = repo_root.join(gate.artifact_path());
    let artifact_sha = hash_file(&artifact, "self-build compiler artifact")?;
    if artifact_sha != gate.artifact_sha256() {
        return Err(format!(
            "artifact SHA-256 mismatch: expected={} actual={artifact_sha}",
            gate.artifact_sha256()
        ));
    }

    let revision = git_output(repo_root, &["rev-parse", "HEAD"])?;
    if revision != gate.source_revision() {
        return Err(format!(
            "source revision mismatch: expected={} actual={revision}",
            gate.source_revision()
        ));
    }

    let status = git_output(repo_root, &["status", "--porcelain"])?;
    if !status.is_empty() {
        return Err("source tree is dirty; compiler-stage gate rejects stale evidence".into());
    }

    Ok(())
}
