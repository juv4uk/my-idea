use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    Current,
    Historical,
    Adr,
    Experiment,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DocumentationSource {
    pub repo: String,
    pub root: PathBuf,
    pub commit_sha: String,
}

impl DocumentationSource {
    pub fn new(
        repo: impl Into<String>,
        root: PathBuf,
        commit_sha: impl Into<String>,
    ) -> Self {
        Self {
            repo: repo.into(),
            root,
            commit_sha: commit_sha.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRecord {
    pub repo: String,
    pub path: String,
    pub title: String,
    pub commit_sha: String,
    pub language: String,
    pub category: String,
    pub status: DocumentStatus,
    pub source_kind: String,
}

#[derive(Debug, Clone)]
struct IndexedDocument {
    record: DocumentRecord,
    searchable: String,
}

#[derive(Debug, Clone, Default)]
pub struct DocumentationIndex {
    documents: Vec<IndexedDocument>,
}

impl DocumentationIndex {
    pub fn build(sources: &[DocumentationSource]) -> Self {
        let mut documents = Vec::new();

        for source in sources {
            if !source.root.is_dir() {
                continue;
            }

            let mut candidates = Vec::new();
            let readme = source.root.join("README.md");
            if readme.is_file() {
                candidates.push(readme);
            }
            collect_markdown(&source.root.join("docs"), &mut candidates);
            candidates.sort_by_key(|path| relative_text(path, &source.root));

            for path in candidates {
                let Ok(body) = fs::read_to_string(&path) else {
                    continue;
                };
                let relative = relative_text(&path, &source.root);
                let record = DocumentRecord {
                    repo: source.repo.clone(),
                    path: relative.clone(),
                    title: title_from_markdown(&body, &path),
                    commit_sha: source.commit_sha.clone(),
                    language: language_from_path(&path).to_string(),
                    category: category_from_path(&relative),
                    status: status_from_document(&relative, &body),
                    source_kind: source_kind_from_path(&path).to_string(),
                };
                let searchable = format!(
                    "{}\n{}\n{}\n{}",
                    record.repo, record.path, record.title, body
                )
                .to_lowercase();
                documents.push(IndexedDocument { record, searchable });
            }
        }

        documents.sort_by(|left, right| {
            (&left.record.repo, &left.record.path).cmp(&(&right.record.repo, &right.record.path))
        });
        Self { documents }
    }

    pub fn records(&self) -> Vec<DocumentRecord> {
        self.documents
            .iter()
            .map(|document| document.record.clone())
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<DocumentRecord> {
        let query = query.trim().to_lowercase();
        self.documents
            .iter()
            .filter(|document| query.is_empty() || document.searchable.contains(&query))
            .map(|document| document.record.clone())
            .collect()
    }
}

fn collect_markdown(directory: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            collect_markdown(&path, out);
        } else if file_type.is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        {
            out.push(path);
        }
    }
}

fn relative_text(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn title_from_markdown(body: &str, path: &Path) -> String {
    body.lines()
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|title| !title.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| fallback_title(path))
}

fn fallback_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("untitled")
        .to_string()
}

fn language_from_path(path: &Path) -> &'static str {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if name.ends_with(".uk.md") {
        "uk"
    } else if name.ends_with(".en.md") {
        "en"
    } else if name.ends_with(".sa.md") {
        "sa"
    } else {
        "unknown"
    }
}

fn category_from_path(path: &str) -> String {
    if path.eq_ignore_ascii_case("README.md") {
        return "root".to_string();
    }
    let mut components = path.split('/');
    if components.next().is_some_and(|part| part.eq_ignore_ascii_case("docs")) {
        match (components.next(), components.next()) {
            (Some(first), Some(_)) => first.to_string(),
            (Some(_), None) | (None, _) => "docs".to_string(),
        }
    } else {
        "root".to_string()
    }
}

fn source_kind_from_path(path: &Path) -> &'static str {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if name.eq_ignore_ascii_case("README.md") {
        "readme"
    } else if name.to_ascii_uppercase().starts_with("ADR-") {
        "adr"
    } else {
        "markdown"
    }
}

fn status_from_document(path: &str, body: &str) -> DocumentStatus {
    let lower_path = path.to_ascii_lowercase();
    let file_name = lower_path.rsplit('/').next().unwrap_or(&lower_path);

    let path_is_historical = lower_path
        .split('/')
        .any(|component| component == "archive");
    let metadata = body
        .lines()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    let explicitly_historical = metadata.iter().any(|line| {
        line == "status: historical"
            || line == "status: superseded"
            || line == "status: archived"
            || line.starts_with("superseded-by:")
    });
    if path_is_historical || explicitly_historical {
        return DocumentStatus::Historical;
    }

    if file_name.to_ascii_uppercase().starts_with("ADR-") {
        return DocumentStatus::Adr;
    }

    if lower_path.contains("experiment") || lower_path.contains("spike") {
        return DocumentStatus::Experiment;
    }

    let explicitly_current = metadata.iter().any(|line| line == "status: current");
    if file_name.eq_ignore_ascii_case("current.md") || explicitly_current {
        return DocumentStatus::Current;
    }

    DocumentStatus::Unknown
}
