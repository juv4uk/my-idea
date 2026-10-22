//! Сусідні репозиторії екосистеми: їхні `repo.lisp` (Swarm Contract v0.1)
//! розбирає ТОЙ САМИЙ reader pinned SENS — жодного другого парсера й
//! жодних нових залежностей. Лише читання файлів і рендер уже наявного AST.

use std::path::{Path, PathBuf};

use sens::{parse, Expr, ExprKind};

/// Один сусідній репозиторій і його самодекларація (де вона є).
pub struct Sibling {
    pub name: String,
    pub id: String,
    pub role: String,
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

/// Поле `(ключ значення …)` форми `repository`.
fn field(form: &Expr, source: &str) -> Option<(String, String)> {
    let parts = elements(form);
    let key = parts.first().and_then(|e| leaf(e, source))?;
    let value = parts
        .iter()
        .skip(1)
        .filter_map(|e| leaf(e, source))
        .collect::<Vec<_>>()
        .join(" ");
    Some((key, value))
}

/// Витягти `(id …)` і `(role …)` із `(repository …)` через pinned SENS.
pub fn contract_identity(source: &str) -> Option<(String, String)> {
    let forms = parse(source).ok()?;
    for form in &forms {
        let parts = elements(form);
        let head = parts.first().and_then(|e| leaf(e, source));
        if head.as_deref() != Some("repository") {
            continue;
        }
        let mut id = String::new();
        let mut role = String::new();
        for item in parts.iter().skip(1) {
            if let Some((key, value)) = field(item, source) {
                match key.as_str() {
                    "id" => id = value,
                    "role" => role = value,
                    _ => {}
                }
            }
        }
        return Some((id, role));
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
        let mut sibling = Sibling {
            name: name.clone(),
            id: String::new(),
            role: String::new(),
            note: "eco-no-contract",
        };
        match std::fs::read_to_string(path.join("repo.lisp")) {
            Ok(text) => match contract_identity(&text) {
                Some((id, role)) => {
                    sibling.id = id;
                    sibling.role = role;
                    sibling.note = "";
                }
                None => sibling.note = "eco-unreadable",
            },
            Err(_) => sibling.note = "eco-no-contract",
        }
        out.push(sibling);
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

#[cfg(test)]
mod tests {
    use super::contract_identity;

    #[test]
    fn reads_swarm_contract_identity_via_pinned_sens() {
        let source = "(repository\n  (id my-idea)\n  (role wsm-tauri-ide)\n  (exports a b))\n";
        assert_eq!(
            contract_identity(source),
            Some(("my-idea".to_string(), "wsm-tauri-ide".to_string()))
        );
        assert_eq!(contract_identity("(other (id x))"), None);
        assert_eq!(contract_identity("("), None);
    }
}
