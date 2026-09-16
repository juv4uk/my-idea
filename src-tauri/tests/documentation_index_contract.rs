//! Contract for #43 (IDE-HELP-INDEX-1): one deterministic, provenance-
//! preserving documentation index over multiple repositories.

use my_idea_lib::documentation::{
    DocumentRecord, DocumentStatus, DocumentationIndex, DocumentationSource,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct TestRepo {
    root: PathBuf,
}

impl TestRepo {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("my-idea-help-{label}-{nonce}"));
        fs::create_dir_all(root.join("docs")).expect("fixture docs directory should exist");
        Self { root }
    }

    fn write(&self, relative: &str, body: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("fixture parent should exist");
        }
        fs::write(path, body).expect("fixture document should be written");
    }

    fn source(&self, repo: &str, sha: &str) -> DocumentationSource {
        DocumentationSource::new(repo, self.root.clone(), sha)
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn record<'a>(records: &'a [DocumentRecord], repo: &str, path: &str) -> &'a DocumentRecord {
    records
        .iter()
        .find(|record| record.repo == repo && record.path == path)
        .unwrap_or_else(|| panic!("missing record {repo}:{path}"))
}

#[test]
fn search_preserves_exact_repo_path_and_full_sha_provenance() {
    let cml = TestRepo::new("cml-provenance");
    cml.write(
        "docs/abi.md",
        "# CML ABI\n\nThe native ABI contract describes register and call boundaries.\n",
    );
    let sha = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let index = DocumentationIndex::build(&[cml.source("cml", sha)]);
    let hits = index.search("ABI");

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].repo, "cml");
    assert_eq!(hits[0].path, "docs/abi.md");
    assert_eq!(hits[0].commit_sha, sha);
    assert_eq!(hits[0].title, "CML ABI");
}

#[test]
fn same_title_in_two_repositories_stays_two_distinct_resources() {
    let my_lisp = TestRepo::new("my-lisp-duplicate");
    let my_idea = TestRepo::new("my-idea-duplicate");
    my_lisp.write("docs/CANON.md", "# Canon\n\nLanguage authority.\n");
    my_idea.write("docs/CANON.md", "# Canon\n\nIDE integration notes.\n");

    let index = DocumentationIndex::build(&[
        my_lisp.source("my-lisp", "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        my_idea.source("my-idea", "cccccccccccccccccccccccccccccccccccccccc"),
    ]);
    let hits = index.search("Canon");

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].title, "Canon");
    assert_eq!(hits[1].title, "Canon");
    assert_ne!(hits[0].repo, hits[1].repo);
    assert_ne!(hits[0].commit_sha, hits[1].commit_sha);
}

#[test]
fn status_and_language_classification_are_conservative_and_fail_closed() {
    let repo = TestRepo::new("classification");
    repo.write("docs/archive/OLD.md", "# Old\n\nStatus: current\n");
    repo.write("docs/ADR-007-BOUNDARY.md", "# Boundary ADR\n");
    repo.write("docs/CURRENT.md", "# Current docs\n");
    repo.write("docs/experiment-spike.md", "# Spike\n");
    repo.write("docs/guide.uk.md", "# Посібник\n");
    repo.write("docs/plain.md", "# Plain\n");

    let index = DocumentationIndex::build(&[
        repo.source("my-idea", "dddddddddddddddddddddddddddddddddddddddd"),
    ]);
    let records = index.records();

    assert_eq!(record(&records, "my-idea", "docs/archive/OLD.md").status, DocumentStatus::Historical);
    assert_ne!(record(&records, "my-idea", "docs/archive/OLD.md").status, DocumentStatus::Current);
    assert_eq!(record(&records, "my-idea", "docs/ADR-007-BOUNDARY.md").status, DocumentStatus::Adr);
    assert_eq!(record(&records, "my-idea", "docs/CURRENT.md").status, DocumentStatus::Current);
    assert_eq!(record(&records, "my-idea", "docs/experiment-spike.md").status, DocumentStatus::Experiment);
    assert_eq!(record(&records, "my-idea", "docs/guide.uk.md").language, "uk");
    assert_eq!(record(&records, "my-idea", "docs/plain.md").language, "unknown");
    assert_eq!(record(&records, "my-idea", "docs/plain.md").status, DocumentStatus::Unknown);
}

#[test]
fn root_readme_and_nested_markdown_are_indexed_but_non_markdown_is_not() {
    let repo = TestRepo::new("scope");
    repo.write("README.md", "# Root help\n");
    repo.write("docs/compiler/lowering.md", "# Lowering\n");
    repo.write("docs/compiler/raw.txt", "not markdown\n");

    let index = DocumentationIndex::build(&[
        repo.source("cml", "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
    ]);
    let records = index.records();

    assert_eq!(record(&records, "cml", "README.md").source_kind, "readme");
    assert_eq!(record(&records, "cml", "README.md").category, "root");
    assert_eq!(record(&records, "cml", "docs/compiler/lowering.md").category, "compiler");
    assert!(!records.iter().any(|item| item.path.ends_with("raw.txt")));
}

#[test]
fn full_text_search_and_rebuild_order_are_deterministic() {
    let repo = TestRepo::new("deterministic");
    repo.write("docs/z-last.md", "# Last\n\nquantum bridge token\n");
    repo.write("docs/a-first.md", "# First\n\nquantum bridge token\n");

    let sources = [repo.source("my-lisp", "ffffffffffffffffffffffffffffffffffffffff")];
    let first = DocumentationIndex::build(&sources);
    let second = DocumentationIndex::build(&sources);

    assert_eq!(first.records(), second.records());
    assert_eq!(first.search("quantum bridge"), second.search("quantum bridge"));
    assert_eq!(first.search(""), first.records());

    let hits = first.search("quantum bridge");
    let paths: Vec<&str> = hits.iter().map(|item| item.path.as_str()).collect();
    assert_eq!(paths, vec!["docs/a-first.md", "docs/z-last.md"]);
}

#[test]
fn changing_only_source_sha_changes_provenance_not_repo_path_or_title() {
    let repo = TestRepo::new("sha-change");
    repo.write("docs/topic.md", "# Stable Topic\n\nSame body.\n");

    let old = DocumentationIndex::build(&[
        repo.source("my-lisp", "1111111111111111111111111111111111111111"),
    ]);
    let new = DocumentationIndex::build(&[
        repo.source("my-lisp", "2222222222222222222222222222222222222222"),
    ]);

    let old_records = old.records();
    let new_records = new.records();
    let old_record = &old_records[0];
    let new_record = &new_records[0];
    assert_eq!(old_record.repo, new_record.repo);
    assert_eq!(old_record.path, new_record.path);
    assert_eq!(old_record.title, new_record.title);
    assert_ne!(old_record.commit_sha, new_record.commit_sha);
}

#[test]
fn missing_source_root_yields_no_guessed_records() {
    let missing = std::env::temp_dir().join(format!(
        "my-idea-help-missing-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(!Path::new(&missing).exists());

    let index = DocumentationIndex::build(&[DocumentationSource::new(
        "cml",
        missing,
        "3333333333333333333333333333333333333333",
    )]);
    assert!(index.records().is_empty());
}
