use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoInfo {
    pub name: String,
    pub path: String,
    pub found: bool,
    pub branch: Option<String>,
    pub sha: Option<String>,
}

fn git_output(repo: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Reports branch + short SHA for a locally cloned ecosystem repo, if present.
/// Повідомляє branch + короткий SHA для локально клонованого репо екосистеми, якщо він є.
pub fn repo_info(name: &str, path: &Path) -> RepoInfo {
    let found = path.join(".git").exists();
    let (branch, sha) = if found {
        (
            git_output(path, &["rev-parse", "--abbrev-ref", "HEAD"]),
            git_output(path, &["rev-parse", "--short", "HEAD"]),
        )
    } else {
        (None, None)
    };
    RepoInfo {
        name: name.to_string(),
        path: path.to_string_lossy().into_owned(),
        found,
        branch,
        sha,
    }
}

/// Reads the pinned git SHA for `package_name` out of this workspace's
/// `Cargo.lock` — a `[[package]] name = ".." source = "git+...#SHA"` block.
/// This is the commit the *embedded* desktop engine (`evaluate_my_lisp`)
/// was actually built against, which can drift from whatever's checked out
/// in the sibling `my-lisp` repo on disk (see AGENTS.md's "two different
/// my-lisp" note) — surfaced so that drift is visible, not silently assumed
/// away. String-scanned rather than pulling in a TOML crate for one field.
///
/// Читає закріплений git SHA для `package_name` з `Cargo.lock` цього
/// workspace — той коміт, проти якого реально зібраний embedded-двигун
/// (`evaluate_my_lisp`), який може розійтись із checkout сусіднього
/// репо `my-lisp` на диску.
pub fn embedded_dependency_sha(workspace_root: &Path, package_name: &str) -> Option<String> {
    let lockfile = fs::read_to_string(workspace_root.join("Cargo.lock")).ok()?;
    let mut lines = lockfile.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "[[package]]" {
            continue;
        }
        let name_line = lines.next()?;
        if name_line.trim() != format!("name = \"{package_name}\"") {
            continue;
        }
        for candidate in lines.by_ref().take(6) {
            if let Some(source) = candidate.trim().strip_prefix("source = \"") {
                return source
                    .trim_end_matches('"')
                    .rsplit('#')
                    .next()
                    .map(|sha| sha.chars().take(7).collect());
            }
            if candidate.trim().is_empty() {
                break;
            }
        }
        return None;
    }
    None
}
