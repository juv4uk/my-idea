//! RED contract for #44 (IDE-HELP-LISP-API-1): canonical Lisp help must
//! combine upstream my-lisp identity/tooling metadata with #43 documentation
//! provenance without inventing another semantic table in my-idea.

use my_idea_lib::documentation::{DocumentationIndex, DocumentationSource};
use my_idea_lib::help_api::{HelpCatalog, HelpRegistry};
use my_idea_lib::ReplSession;
use my_lisp::language_items;
use std::fs;
use std::path::PathBuf;
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
        let root = std::env::temp_dir().join(format!("my-idea-help-api-{label}-{nonce}"));
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

#[test]
fn topic_identity_signature_and_peer_surfaces_come_from_my_lisp() {
    let items = language_items();
    let canonical = items
        .iter()
        .find(|item| item.name == "car" && item.semantic_id.is_some())
        .expect("pinned my-lisp should expose car through language_items");
    let identity = canonical
        .semantic_id
        .expect("car must have upstream semantic identity");
    let peer = items
        .iter()
        .find(|item| item.semantic_id == Some(identity) && item.name != canonical.name)
        .expect("semantic registry should expose at least one peer surface for this witness");

    let catalog = HelpCatalog::new(DocumentationIndex::default());
    let topic = catalog
        .topic(&canonical.name)
        .expect("canonical surface should resolve");
    let peer_topic = catalog.topic(&peer.name).expect("peer surface should resolve");
    let id_topic = catalog
        .topic(identity)
        .expect("semantic identity itself should resolve to the same topic");

    assert_eq!(topic.identity, identity);
    assert_eq!(peer_topic.identity, identity);
    assert_eq!(id_topic.identity, identity);
    assert_eq!(topic.signature, canonical.signature);
    assert_eq!(topic.summary, canonical.documentation);
    assert!(topic.surfaces.contains(&canonical.name));
    assert!(topic.surfaces.contains(&peer.name));
    assert_eq!(topic.identity, peer_topic.identity);
    assert_eq!(topic.identity, id_topic.identity);
}

#[test]
fn document_search_preserves_repo_path_and_full_sha() {
    let cml = TestRepo::new("search");
    cml.write(
        "docs/abi.md",
        "# CML ABI\n\nMachine code ABI and calling convention.\n",
    );
    let sha = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let index = DocumentationIndex::build(&[cml.source("cml", sha)]);
    let catalog = HelpCatalog::new(index);

    let hits = catalog.search("ABI");

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].repo, "cml");
    assert_eq!(hits[0].path, "docs/abi.md");
    assert_eq!(hits[0].commit_sha, sha);
}

#[test]
fn unknown_topic_fails_closed_without_fuzzy_guessing() {
    let catalog = HelpCatalog::new(DocumentationIndex::default());
    assert!(catalog.topic("this-topic-does-not-exist").is_none());
}

#[test]
fn installed_lisp_help_api_returns_native_data_for_topic_and_search() {
    let items = language_items();
    let canonical = items
        .iter()
        .find(|item| item.name == "car" && item.semantic_id.is_some())
        .expect("pinned my-lisp should expose car through language_items");
    let identity = canonical
        .semantic_id
        .expect("car must have upstream semantic identity");
    let peer = items
        .iter()
        .find(|item| item.semantic_id == Some(identity) && item.name != canonical.name)
        .expect("semantic registry should expose a peer surface");

    let cml = TestRepo::new("lisp-search");
    cml.write("docs/abi.md", "# CML ABI\n\nNative ABI witness.\n");
    let sha = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let index = DocumentationIndex::build(&[cml.source("cml", sha)]);

    let registry = HelpRegistry::new(HelpCatalog::new(index));
    let mut repl = ReplSession::default();
    registry.install_into(&mut repl);

    let topic = repl
        .evaluate("(help/topic 'car)")
        .expect("help/topic should evaluate in the installed live session");
    let peer_topic = repl
        .evaluate(&format!("(help/topic '{})", peer.name))
        .expect("peer surface should reach the same help capability");
    let id_topic = repl
        .evaluate(&format!("(help/topic \"{identity}\")"))
        .expect("semantic ID should reach the same help capability");
    assert!(
        topic.value.starts_with('('),
        "help/topic must return Lisp data, not an opaque JSON string: {}",
        topic.value
    );
    assert!(topic.value.contains(identity));
    assert!(peer_topic.value.contains(identity));
    assert!(id_topic.value.contains(identity));
    assert!(!topic.value.trim_start().starts_with("{\""));

    let search = repl
        .evaluate("(help/search \"ABI\")")
        .expect("help/search should evaluate through the same installed API");
    assert!(search.value.contains("docs/abi.md"));
    assert!(search.value.contains(sha));

    let source = repl
        .evaluate("(help/source 'car)")
        .expect("help/source should be available as Lisp data");
    assert!(source.value.starts_with('('));

    let related = repl
        .evaluate("(help/related 'car)")
        .expect("help/related should fail conservatively rather than guess");
    assert_eq!(related.value, "()");

    let unknown = repl
        .evaluate("(help/topic 'this-topic-does-not-exist)")
        .expect("unknown help topic should fail closed as data");
    assert_eq!(unknown.value, "()");
}
