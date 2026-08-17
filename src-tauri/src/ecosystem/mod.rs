mod contracts;
mod evidence;
mod git;

use contracts::{CmlCompatibility, CmlStatus, IsaContract, LanguageContract};
use evidence::RequirementRow;
use git::RepoInfo;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityCheck {
    pub language_match: bool,
    pub isa_match: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EcosystemStatus {
    pub my_lisp: RepoInfo,
    pub my_lisp_contract: Option<LanguageContract>,
    pub fpga_lisp: RepoInfo,
    pub fpga_lisp_contract: Option<IsaContract>,
    pub cml: RepoInfo,
    pub cml_compatibility: Option<CmlCompatibility>,
    pub compatibility: Option<CompatibilityCheck>,
    pub cml_status: Option<CmlStatus>,
    pub evidence_matrix: Vec<RequirementRow>,
    /// The git SHA the desktop app's own embedded my-lisp engine
    /// (`evaluate_my_lisp`) was built against — from this workspace's
    /// `Cargo.lock`, not the sibling `my-lisp` repo's current checkout.
    pub embedded_my_lisp_sha: Option<String>,
}

/// The three ecosystem repos are expected as siblings of `my-idea` on disk, e.g.
/// `~/Documents/GitHub/{my-idea,my-lisp,fpga-lisp,cml}` — offline-first, no GitHub API.
/// Три репо екосистеми очікуються поруч із `my-idea` на диску, офлайн-first, без GitHub API.
fn my_idea_root() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")); // .../my-idea/src-tauri
    manifest_dir.parent().map(Path::to_path_buf) // .../my-idea
}

fn siblings_root() -> Option<PathBuf> {
    let my_idea_dir = my_idea_root()?;
    let github_root = my_idea_dir.parent()?; // .../GitHub
    Some(github_root.to_path_buf())
}

/// Builds the current ecosystem-wide status by scanning sibling repos and their
/// machine-readable contracts. Called fresh on every request — no caching, so
/// the result always reflects the on-disk state (`Run ecosystem check`).
/// Будує поточний стан екосистеми, скануючи сусідні репо та їхні
/// машинно-читані контракти. Викликається щоразу заново — без кешування.
pub fn status() -> EcosystemStatus {
    let root = siblings_root();

    let my_lisp_path = root
        .as_ref()
        .map(|r| r.join("my-lisp"))
        .unwrap_or_default();
    let fpga_lisp_path = root
        .as_ref()
        .map(|r| r.join("fpga-lisp"))
        .unwrap_or_default();
    let cml_path = root.as_ref().map(|r| r.join("cml")).unwrap_or_default();

    let my_lisp = git::repo_info("my-lisp", &my_lisp_path);
    let fpga_lisp = git::repo_info("fpga-lisp", &fpga_lisp_path);
    let cml = git::repo_info("cml", &cml_path);

    let my_lisp_contract = contracts::read_language_contract(&my_lisp_path);
    let fpga_lisp_contract = contracts::read_isa_contract(&fpga_lisp_path);
    let cml_compatibility = contracts::read_cml_compatibility(&cml_path);
    let cml_status = contracts::read_cml_status(&my_lisp_path);

    let mut evidence_records = evidence::scan(&my_lisp_path);
    evidence_records.extend(evidence::scan(&fpga_lisp_path));
    evidence_records.extend(evidence::scan(&cml_path));
    let evidence_matrix = evidence::matrix(evidence_records);

    let embedded_my_lisp_sha = my_idea_root()
        .and_then(|root| git::embedded_dependency_sha(&root, "my-lisp"));

    let compatibility = cml_compatibility.as_ref().map(|cml_compat| {
        let language_match = my_lisp_contract
            .as_ref()
            .map(|c| c.version.major == cml_compat.language_contract.major
                && c.version.minor == cml_compat.language_contract.minor)
            .unwrap_or(false);
        let isa_match = fpga_lisp_contract
            .as_ref()
            .map(|c| c.version.major == cml_compat.isa_contract.major
                && c.version.minor == cml_compat.isa_contract.minor)
            .unwrap_or(false);
        CompatibilityCheck {
            language_match,
            isa_match,
        }
    });

    EcosystemStatus {
        my_lisp,
        my_lisp_contract,
        fpga_lisp,
        fpga_lisp_contract,
        cml,
        cml_compatibility,
        compatibility,
        cml_status,
        evidence_matrix,
        embedded_my_lisp_sha,
    }
}
