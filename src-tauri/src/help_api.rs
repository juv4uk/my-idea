//! Lisp-facing Help API for my-idea (#44).
//!
//! Authority is deliberately split rather than copied here:
//! - `my_lisp::language_items()` owns language identity/tooling metadata;
//! - `DocumentationIndex` owns repository document provenance/search;
//! - this module only adapts those sources into native Lisp data.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

use my_lisp::{
    eval_expr, exact_arity, language_items, register_capability, Environment, ErrorKind, Expr,
    LanguageError, LanguageItemKind, Span, Value,
};

use crate::documentation::{DocumentRecord, DocumentStatus, DocumentationIndex};
use crate::ReplSession;

const TOKEN_ENV_KEY: &str = "%help-registry-token%";
const HANDLE_KIND: &str = "help-registry";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpTopic {
    pub identity: String,
    pub kind: String,
    pub signature: String,
    pub surfaces: Vec<String>,
    pub summary: String,
    pub documents: Vec<DocumentRecord>,
}

#[derive(Debug, Clone, Default)]
pub struct HelpCatalog {
    documents: DocumentationIndex,
}

impl HelpCatalog {
    pub fn new(documents: DocumentationIndex) -> Self {
        Self { documents }
    }

    pub fn topic(&self, query: &str) -> Option<HelpTopic> {
        let items = language_items();
        let matched = items
            .iter()
            .find(|item| item.name == query || item.semantic_id == Some(query))?;
        let identity = matched.semantic_id?;

        let mut surfaces = items
            .iter()
            .filter(|item| item.semantic_id == Some(identity))
            .map(|item| item.name.clone())
            .collect::<Vec<_>>();
        surfaces.sort();
        surfaces.dedup();

        let documents = self
            .documents
            .search(identity)
            .into_iter()
            .filter(|record| record.repo == "my-lisp")
            .collect();

        Some(HelpTopic {
            identity: identity.to_string(),
            kind: language_item_kind(matched.kind).to_string(),
            signature: matched.signature.to_string(),
            surfaces,
            summary: matched.documentation.to_string(),
            documents,
        })
    }

    pub fn search(&self, query: &str) -> Vec<DocumentRecord> {
        self.documents.search(query)
    }

    pub fn source(&self, query: &str) -> Vec<DocumentRecord> {
        self.topic(query)
            .map(|topic| topic.documents)
            .unwrap_or_default()
    }
}

fn language_item_kind(kind: LanguageItemKind) -> &'static str {
    match kind {
        LanguageItemKind::Builtin => "builtin",
        LanguageItemKind::Macro => "macro",
        LanguageItemKind::SyntaxForm => "syntax-form",
    }
}

thread_local! {
    static CATALOGS: RefCell<HashMap<u64, Rc<HelpCatalog>>> = RefCell::new(HashMap::new());
    static NEXT_TOKEN: Cell<u64> = Cell::new(1);
}

fn catalog_for(token: u64) -> Rc<HelpCatalog> {
    CATALOGS.with(|catalogs| {
        catalogs
            .borrow()
            .get(&token)
            .cloned()
            .expect("help registry token must be registered before use")
    })
}

fn token_from_environment(environment: &Environment, span: Span) -> Result<u64, LanguageError> {
    match environment.get(TOKEN_ENV_KEY).as_ref() {
        Some(Value::HostHandle { kind, token }) if &**kind == HANDLE_KIND => Ok(*token),
        _ => Err(LanguageError::new(
            ErrorKind::UnknownSymbol,
            "help API used outside a HelpRegistry-installed session",
            span,
        )),
    }
}

fn expect_query(value: Value, what: &str, span: Span) -> Result<String, LanguageError> {
    match value {
        Value::String(text) | Value::Symbol(text) => Ok(text.to_string()),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            format!("{what} expects a symbol or string"),
            span,
        )),
    }
}

fn help_topic(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("help/topic", arguments, 1, span)?;
    let token = token_from_environment(environment, span)?;
    let query = expect_query(
        eval_expr(&arguments[0], environment)?,
        "help/topic",
        arguments[0].span,
    )?;
    Ok(catalog_for(token)
        .topic(&query)
        .map(topic_value)
        .unwrap_or(Value::Nil))
}

fn help_search(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("help/search", arguments, 1, span)?;
    let token = token_from_environment(environment, span)?;
    let query = expect_query(
        eval_expr(&arguments[0], environment)?,
        "help/search",
        arguments[0].span,
    )?;
    Ok(list_value(
        catalog_for(token)
            .search(&query)
            .into_iter()
            .map(document_value)
            .collect(),
    ))
}

fn help_source(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("help/source", arguments, 1, span)?;
    let token = token_from_environment(environment, span)?;
    let query = expect_query(
        eval_expr(&arguments[0], environment)?,
        "help/source",
        arguments[0].span,
    )?;
    Ok(list_value(
        catalog_for(token)
            .source(&query)
            .into_iter()
            .map(document_value)
            .collect(),
    ))
}

fn help_related(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("help/related", arguments, 1, span)?;
    token_from_environment(environment, span)?;
    // v0 is deliberately conservative: no source currently owns explicit
    // semantic relations, so returning an empty list is more honest than
    // guessing neighbours from spelling or documentation text.
    let _ = expect_query(
        eval_expr(&arguments[0], environment)?,
        "help/related",
        arguments[0].span,
    )?;
    Ok(Value::Nil)
}

fn ensure_capabilities_installed() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        register_capability("help/topic", help_topic);
        register_capability("help/search", help_search);
        register_capability("help/source", help_source);
        register_capability("help/related", help_related);
    });
}

pub struct HelpRegistry {
    token: u64,
}

impl HelpRegistry {
    pub fn new(catalog: HelpCatalog) -> Self {
        let token = NEXT_TOKEN.with(|next| {
            let token = next.get();
            next.set(token + 1);
            token
        });
        CATALOGS.with(|catalogs| {
            catalogs.borrow_mut().insert(token, Rc::new(catalog));
        });
        Self { token }
    }

    pub fn install_into(&self, repl: &mut ReplSession) {
        ensure_capabilities_installed();
        repl.environment().define(
            TOKEN_ENV_KEY,
            Value::HostHandle {
                kind: Rc::from(HANDLE_KIND),
                token: self.token,
            },
        );
    }
}

impl Drop for HelpRegistry {
    fn drop(&mut self) {
        CATALOGS.with(|catalogs| {
            catalogs.borrow_mut().remove(&self.token);
        });
    }
}

fn topic_value(topic: HelpTopic) -> Value {
    alist_value(vec![
        ("identity", string_value(topic.identity)),
        ("kind", string_value(topic.kind)),
        ("signature", string_value(topic.signature)),
        (
            "surfaces",
            list_value(topic.surfaces.into_iter().map(string_value).collect()),
        ),
        ("summary", string_value(topic.summary)),
        (
            "documents",
            list_value(topic.documents.into_iter().map(document_value).collect()),
        ),
    ])
}

fn document_value(record: DocumentRecord) -> Value {
    alist_value(vec![
        ("repo", string_value(record.repo)),
        ("path", string_value(record.path)),
        ("title", string_value(record.title)),
        ("commit-sha", string_value(record.commit_sha)),
        ("language", string_value(record.language)),
        ("category", string_value(record.category)),
        ("status", string_value(document_status(record.status))),
        ("source-kind", string_value(record.source_kind)),
    ])
}

fn document_status(status: DocumentStatus) -> &'static str {
    match status {
        DocumentStatus::Current => "current",
        DocumentStatus::Historical => "historical",
        DocumentStatus::Adr => "adr",
        DocumentStatus::Experiment => "experiment",
        DocumentStatus::Unknown => "unknown",
    }
}

fn alist_value(fields: Vec<(&'static str, Value)>) -> Value {
    list_value(
        fields
            .into_iter()
            .map(|(name, value)| list_value(vec![Value::Symbol(Rc::from(name)), value]))
            .collect(),
    )
}

fn list_value(values: Vec<Value>) -> Value {
    values.into_iter().rev().fold(Value::Nil, |tail, head| {
        Value::Pair(Rc::new(head), Rc::new(tail))
    })
}

fn string_value(text: impl AsRef<str>) -> Value {
    Value::String(Rc::from(text.as_ref()))
}
