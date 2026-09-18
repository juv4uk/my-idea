//! Deterministic, inspectable self-build planning for issue #12.
//!
//! This module owns orchestration/provenance data only. Building a plan never
//! starts CML, Bun, Cargo, Tauri, or a generated artifact. Execution begins in
//! issue #13.

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt;

pub const SELF_BUILD_SCHEMA: &str = "self-build-plan-v1";
pub const SELF_BUILD_TARGET: &str = "x86_64-linux";

const PROJECT_SOURCE: &str = "self-build/my-idea.lisp";
const COMPILER_ARTIFACT: &str = "target/self-build/my-idea-x86_64-linux";
const PLATFORM_OUTPUT: &str = "src-tauri/target/release/bundle";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfBuildInputs {
    source_revision: String,
    source_clean: bool,
    my_lisp_revision: String,
    compiler: String,
    compiler_revision: String,
    compiler_target: String,
    rustc_version: String,
    cargo_version: String,
    bun_version: String,
    tauri_version: String,
}

impl SelfBuildInputs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_revision: impl Into<String>,
        source_clean: bool,
        my_lisp_revision: impl Into<String>,
        compiler: impl Into<String>,
        compiler_revision: impl Into<String>,
        compiler_target: impl Into<String>,
        rustc_version: impl Into<String>,
        cargo_version: impl Into<String>,
        bun_version: impl Into<String>,
        tauri_version: impl Into<String>,
    ) -> Self {
        Self {
            source_revision: source_revision.into(),
            source_clean,
            my_lisp_revision: my_lisp_revision.into(),
            compiler: compiler.into(),
            compiler_revision: compiler_revision.into(),
            compiler_target: compiler_target.into(),
            rustc_version: rustc_version.into(),
            cargo_version: cargo_version.into(),
            bun_version: bun_version.into(),
            tauri_version: tauri_version.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourcePlan {
    root: &'static str,
    revision: String,
    clean: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct LanguageRuntimePlan {
    name: &'static str,
    revision: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompilerPlan {
    executable: String,
    revision: String,
    target: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolchainPlan {
    rustc: String,
    cargo: String,
    bun: String,
    tauri: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildCommand {
    executable: String,
    args: Vec<String>,
}

impl BuildCommand {
    fn new(executable: impl Into<String>, args: &[&str]) -> Self {
        Self {
            executable: executable.into(),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildStage {
    name: &'static str,
    role: &'static str,
    command: BuildCommand,
    inputs: Vec<&'static str>,
    outputs: Vec<&'static str>,
    bootstrap_platform: bool,
}

impl BuildStage {
    fn compiler() -> Self {
        Self {
            name: "compiler",
            role: "authoritative-project-compiler",
            command: BuildCommand::new(
                "cml-compile",
                &["x86-elf", PROJECT_SOURCE, COMPILER_ARTIFACT],
            ),
            inputs: vec![PROJECT_SOURCE],
            outputs: vec![COMPILER_ARTIFACT],
            bootstrap_platform: false,
        }
    }

    fn frontend() -> Self {
        Self {
            name: "frontend",
            role: "wasm-clojurescript-frontend",
            command: BuildCommand::new("bun", &["run", "build"]),
            inputs: vec!["external/my-lisp", "src-cljs", "public"],
            outputs: vec!["dist"],
            bootstrap_platform: false,
        }
    }

    fn platform() -> Self {
        Self {
            name: "platform",
            role: "tauri-bootstrap-platform",
            command: BuildCommand::new("bun", &["run", "tauri", "build"]),
            inputs: vec![COMPILER_ARTIFACT, "dist", "src-tauri"],
            outputs: vec![PLATFORM_OUTPUT],
            bootstrap_platform: true,
        }
    }

    pub fn bootstrap_platform(&self) -> bool {
        self.bootstrap_platform
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SelfBuildPlanBody {
    schema_version: &'static str,
    source: SourcePlan,
    language_runtime: LanguageRuntimePlan,
    compiler: CompilerPlan,
    toolchain: ToolchainPlan,
    stages: Vec<BuildStage>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfBuildPlan {
    schema_version: &'static str,
    source: SourcePlan,
    language_runtime: LanguageRuntimePlan,
    compiler: CompilerPlan,
    toolchain: ToolchainPlan,
    stages: Vec<BuildStage>,
    digest: String,
}

impl SelfBuildPlan {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn source_revision(&self) -> &str {
        &self.source.revision
    }

    pub fn my_lisp_revision(&self) -> &str {
        &self.language_runtime.revision
    }

    pub fn compiler_revision(&self) -> &str {
        &self.compiler.revision
    }

    pub fn compiler_target(&self) -> &str {
        &self.compiler.target
    }

    pub fn stage_names(&self) -> Vec<&str> {
        self.stages.iter().map(|stage| stage.name).collect()
    }

    pub fn stage_commands(&self) -> Vec<(&str, Vec<&str>)> {
        self.stages
            .iter()
            .map(|stage| {
                (
                    stage.command.executable.as_str(),
                    stage.command.args.iter().map(String::as_str).collect(),
                )
            })
            .collect()
    }

    pub fn platform_stage(&self) -> &BuildStage {
        self.stages
            .iter()
            .find(|stage| stage.name == "platform")
            .expect("self-build v1 always has a platform stage")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelfBuildPlanError {
    InvalidProvenance(String),
    UnsupportedTarget(String),
    Serialization(String),
}

impl fmt::Display for SelfBuildPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProvenance(message)
            | Self::UnsupportedTarget(message)
            | Self::Serialization(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for SelfBuildPlanError {}

fn require_sha(label: &str, value: &str) -> Result<(), SelfBuildPlanError> {
    if value.len() != 40 || !value.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(SelfBuildPlanError::InvalidProvenance(format!(
            "{label} must be a full 40-hex git revision"
        )));
    }
    Ok(())
}

fn require_value(label: &str, value: &str) -> Result<(), SelfBuildPlanError> {
    if value.trim().is_empty() || value.trim().eq_ignore_ascii_case("unavailable") {
        return Err(SelfBuildPlanError::InvalidProvenance(format!(
            "{label} provenance must be explicit"
        )));
    }
    Ok(())
}

pub fn build_self_build_plan(
    inputs: SelfBuildInputs,
) -> Result<SelfBuildPlan, SelfBuildPlanError> {
    require_sha("source revision", &inputs.source_revision)?;
    require_sha("my-lisp revision", &inputs.my_lisp_revision)?;
    require_sha("compiler revision", &inputs.compiler_revision)?;

    if !inputs.source_clean {
        return Err(SelfBuildPlanError::InvalidProvenance(
            "source tree is dirty; self-build plan requires a clean checkout".into(),
        ));
    }

    if inputs.compiler != "cml-compile" {
        return Err(SelfBuildPlanError::InvalidProvenance(
            "authoritative compiler must be cml-compile".into(),
        ));
    }

    if inputs.compiler_target != SELF_BUILD_TARGET {
        return Err(SelfBuildPlanError::UnsupportedTarget(format!(
            "self-build plan v1 supports only {SELF_BUILD_TARGET}, got {}",
            inputs.compiler_target
        )));
    }

    require_value("rustc", &inputs.rustc_version)?;
    require_value("cargo", &inputs.cargo_version)?;
    require_value("bun", &inputs.bun_version)?;
    require_value("tauri", &inputs.tauri_version)?;

    let body = SelfBuildPlanBody {
        schema_version: SELF_BUILD_SCHEMA,
        source: SourcePlan {
            root: ".",
            revision: inputs.source_revision,
            clean: true,
        },
        language_runtime: LanguageRuntimePlan {
            name: "my-lisp",
            revision: inputs.my_lisp_revision,
        },
        compiler: CompilerPlan {
            executable: inputs.compiler,
            revision: inputs.compiler_revision,
            target: inputs.compiler_target,
        },
        toolchain: ToolchainPlan {
            rustc: inputs.rustc_version,
            cargo: inputs.cargo_version,
            bun: inputs.bun_version,
            tauri: inputs.tauri_version,
        },
        stages: vec![
            BuildStage::compiler(),
            BuildStage::frontend(),
            BuildStage::platform(),
        ],
    };

    let canonical_body = serde_json::to_vec(&body)
        .map_err(|error| SelfBuildPlanError::Serialization(error.to_string()))?;
    let digest = format!("{:x}", Sha256::digest(canonical_body));

    Ok(SelfBuildPlan {
        schema_version: body.schema_version,
        source: body.source,
        language_runtime: body.language_runtime,
        compiler: body.compiler,
        toolchain: body.toolchain,
        stages: body.stages,
        digest,
    })
}

pub fn canonical_plan_json(plan: &SelfBuildPlan) -> Result<String, SelfBuildPlanError> {
    serde_json::to_string(plan)
        .map_err(|error| SelfBuildPlanError::Serialization(error.to_string()))
}
