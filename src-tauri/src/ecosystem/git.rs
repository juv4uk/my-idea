use serde::Serialize;
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
