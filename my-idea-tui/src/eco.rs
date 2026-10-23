//! Сусідні репозиторії екосистеми та їхній Swarm Contract (`repo.lisp`),
//! розібраний ТИМ САМИМ reader pinned SENS — жодного другого парсера й
//! жодних нових залежностей. Лише читання файлів і рендер уже наявного AST.

use std::path::{Path, PathBuf};

use sens::{parse, Expr, ExprKind};

/// Самодекларація одного репозиторію (`(repository …)`).
#[derive(Default, Clone)]
pub struct Sibling {
    pub name: String,
    pub id: String,
    pub role: String,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    /// Порожньо, якщо контракт розібрано; інакше — i18n-ключ стану.
    pub note: &'static str,
}

/// Елементи форми (List/Call/DomainCall), щоб не залежати від конкретного
/// варіанта вузла для кожної форми контракту.
fn elements<'a>(expr: &'a Expr) -> Vec<&'a Expr> {
    match &expr.kind {
        ExprKind::List(items) => items.iter().collect(),
        ExprKind::Call(_, args) | ExprKind::DomainCall(_, args) => args.iter().collect(),
        _ => Vec::new(),
    }
}

/// Написання вузла як у джерелі (лише оригінал, без вигаданих доменних кодів).
fn leaf(expr: &Expr, source: &str) -> Option<String> {
    source
        .get(expr.span.start..expr.span.end)
        .map(|raw| raw.trim().to_owned())
}

/// Поле `(ключ значення …)` форми `repository` — ключ і всі значення.
fn field(form: &Expr, source: &str) -> Option<(String, Vec<String>)> {
    let parts = elements(form);
    let key = parts.first().and_then(|e| leaf(e, source))?;
    let values = parts
        .iter()
        .skip(1)
        .filter_map(|e| leaf(e, source))
        .collect::<Vec<_>>();
    Some((key, values))
}

/// Розібрати `(repository …)` у самодекларацію через pinned SENS.
pub fn contract(source: &str) -> Option<Sibling> {
    let forms = parse(source).ok()?;
    for form in &forms {
        let parts = elements(form);
        if parts.first().and_then(|e| leaf(e, source)).as_deref() != Some("repository") {
            continue;
        }
        let mut sibling = Sibling::default();
        for item in parts.iter().skip(1) {
            if let Some((key, values)) = field(item, source) {
                match key.as_str() {
                    "id" => sibling.id = values.first().cloned().unwrap_or_default(),
                    "role" => sibling.role = values.first().cloned().unwrap_or_default(),
                    "imports" => sibling.imports = values,
                    "exports" => sibling.exports = values,
                    _ => {}
                }
            }
        }
        return Some(sibling);
    }
    None
}

/// Сусідні репозиторії (каталоги поруч із my-idea, що мають `.git` або `repo.lisp`).
pub fn siblings(root: &Path) -> Vec<Sibling> {
    let parent = root.parent().unwrap_or(root);
    let mut out: Vec<Sibling> = Vec::new();
    let entries = match std::fs::read_dir(parent) {
        Ok(entries) => entries,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        if !(path.join(".git").exists() || path.join("repo.lisp").is_file()) {
            continue;
        }
        match std::fs::read_to_string(path.join("repo.lisp")) {
            Ok(text) => match contract(&text) {
                Some(mut sibling) => {
                    sibling.name = name;
                    sibling.note = "";
                    out.push(sibling);
                }
                None => out.push(Sibling {
                    name,
                    note: "eco-unreadable",
                    ..Default::default()
                }),
            },
            Err(_) => out.push(Sibling {
                name,
                note: "eco-no-contract",
                ..Default::default()
            }),
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Ребро графа знань: можливість і хто її експортує / імпортує.
pub struct Capability {
    pub name: String,
    pub exporters: Vec<String>,
    pub importers: Vec<String>,
}

/// Звʼязки між сусідами за спільними можливостями `imports`/`exports`.
pub fn capabilities(siblings: &[Sibling]) -> Vec<Capability> {
    let mut names: Vec<String> = Vec::new();
    for sibling in siblings {
        for capability in sibling.exports.iter().chain(sibling.imports.iter()) {
            if !names.iter().any(|known| known == capability) {
                names.push(capability.clone());
            }
        }
    }
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let mut exporters = Vec::new();
            let mut importers = Vec::new();
            for sibling in siblings {
                if sibling.exports.iter().any(|c| *c == name) {
                    exporters.push(sibling.name.clone());
                }
                if sibling.imports.iter().any(|c| *c == name) {
                    importers.push(sibling.name.clone());
                }
            }
            Capability {
                name,
                exporters,
                importers,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{capabilities, contract, Sibling};

    #[test]
    fn reads_swarm_contract_via_pinned_sens() {
        let source = "(repository\n  (id my-idea)\n  (role wsm-tauri-ide)\n  \
                      (imports language-contract compiler-cli)\n  (exports project-editing))\n";
        let parsed = contract(source).expect("contract parsed");
        assert_eq!(parsed.id, "my-idea");
        assert_eq!(parsed.role, "wsm-tauri-ide");
        assert_eq!(parsed.imports, vec!["language-contract", "compiler-cli"]);
        assert_eq!(parsed.exports, vec!["project-editing"]);
        assert!(contract("(other (id x))").is_none());
        assert!(contract("(").is_none());
    }

    #[test]
    fn derives_capability_edges_between_siblings() {
        let sens = Sibling {
            name: "sens".into(),
            exports: vec!["language-contract".into()],
            ..Default::default()
        };
        let ide = Sibling {
            name: "my-idea".into(),
            imports: vec!["language-contract".into()],
            ..Default::default()
        };
        let edges = capabilities(&[sens, ide]);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].name, "language-contract");
        assert_eq!(edges[0].exporters, vec!["sens"]);
        assert_eq!(edges[0].importers, vec!["my-idea"]);
    }
}
