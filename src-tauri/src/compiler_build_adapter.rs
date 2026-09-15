use crate::{
    compiler_bridge::{CompilerArtifact, CompilerBridge, CompilerDiagnostic, CompilerFailure, CompilerRequest},
    process_service::ProcessSpec,
};

const COMPILED_LISP_PROFILE: &str = "compiled-lisp-artifact";

#[derive(Clone, Debug)]
pub struct CompilerBuildAdapter {
    bridge: CompilerBridge,
}

impl CompilerBuildAdapter {
    pub fn new(bridge: CompilerBridge) -> Self {
        Self { bridge }
    }

    pub fn compile_project(
        &self,
        request: &CompilerRequest,
    ) -> Result<CompiledProject, CompilerFailure> {
        let run = self.bridge.compile_observed(request)?;
        Ok(CompiledProject {
            artifact: run.artifact().clone(),
            diagnostics: run.diagnostics().to_vec(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct CompiledProject {
    artifact: CompilerArtifact,
    diagnostics: Vec<CompilerDiagnostic>,
}

impl CompiledProject {
    pub fn artifact(&self) -> &CompilerArtifact {
        &self.artifact
    }

    pub fn diagnostics(&self) -> &[CompilerDiagnostic] {
        &self.diagnostics
    }

    pub fn process_spec(&self) -> Result<ProcessSpec, String> {
        let executable = self
            .artifact
            .path()
            .canonicalize()
            .map_err(|error| format!("compiled artifact is unavailable: {error}"))?;
        Ok(ProcessSpec {
            profile: COMPILED_LISP_PROFILE.into(),
            executable,
            args: Vec::new(),
        })
    }
}
