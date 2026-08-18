use super::contracts::{as_list, symbol_name};
use my_lisp::{parse, Expr};
use serde::Serialize;
use std::fs;
use std::path::Path;

/// One repository's Swarm Contract v0.1 self-declaration (`repo.my`), per
/// `my-lisp/docs/swarm-mesh-v2.md`. Not every sibling repo has adopted this
/// yet (see `MYIDEA-SWARM-CONTRACT-01` — this repo hasn't either) — that's
/// an absent node, not a parse error, same tolerance as `evidence.rs` for
/// a missing `evidence/` directory.
/// Самодекларація одного репо за Swarm Contract v0.1 (`repo.my`). Не кожне
/// сусіднє репо ще прийняло цю конвенцію — це відсутній вузол, не помилка
/// парсингу.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoNode {
    pub id: String,
    pub found: bool,
    pub role: Option<String>,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
    pub capabilities: Vec<String>,
    pub authorities: Vec<String>,
    pub non_authorities: Vec<String>,
}

impl RepoNode {
    fn missing(id: &str) -> Self {
        RepoNode {
            id: id.to_string(),
            found: false,
            role: None,
            exports: Vec::new(),
            imports: Vec::new(),
            capabilities: Vec::new(),
            authorities: Vec::new(),
            non_authorities: Vec::new(),
        }
    }
}

/// A capability-level edge derived from two repos' own `exports`/`imports`
/// lists — `to` exports `via_capability`, `from` imports it. Coarse: this
/// says nothing about *which* specific claim moved, only that the two
/// repos' declared surfaces overlap. Claim-level edges (phase 2, see
/// `docs/knowledge-graph-design.md`) are the precise version.
/// Грубе ребро, виведене з власних `exports`/`imports` двох репо: `to`
/// експортує `via_capability`, `from` імпортує його.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoEdge {
    pub from: String,
    pub to: String,
    pub via_capability: String,
}

/// Finds `(key value ...)` among `items` — the flat form `repo.my` actually
/// uses (`(exports isa-contract evidence)`, not a dotted alist pair like
/// contract files' `(key . value)`) — and returns the trailing symbol names.
/// Шукає `(key value ...)` серед `items` — плаский формат, який реально
/// використовує `repo.my` — і повертає символи-хвости як рядки.
fn field_symbols(items: &[Expr], key: &str) -> Vec<String> {
    items
        .iter()
        .find_map(|item| {
            let sub = as_list(item)?;
            (symbol_name(sub.first()?)? == key).then(|| {
                sub[1..]
                    .iter()
                    .filter_map(symbol_name)
                    .map(str::to_string)
                    .collect()
            })
        })
        .unwrap_or_default()
}

fn field_symbol(items: &[Expr], key: &str) -> Option<String> {
    field_symbols(items, key).into_iter().next()
}

/// Parses one `repo.my` file. Leading `;` comment lines (fpga-lisp's own
/// convention, see that file) are handled by my-lisp's own reader.
/// Парсить один файл `repo.my`. Провідні `;`-коментарі обробляє власний
/// reader my-lisp.
fn parse_repo_node(id: &str, raw: &str) -> Option<RepoNode> {
    let forms = parse(raw).ok()?;
    let top = forms.into_iter().next()?;
    let items = as_list(&top)?;
    if symbol_name(items.first()?)? != "repository" {
        return None;
    }
    let items = &items[1..];

    Some(RepoNode {
        id: id.to_string(),
        found: true,
        role: field_symbol(items, "role"),
        exports: field_symbols(items, "exports"),
        imports: field_symbols(items, "imports"),
        capabilities: field_symbols(items, "capabilities"),
        authorities: field_symbols(items, "authorities"),
        non_authorities: field_symbols(items, "non-authorities"),
    })
}

/// Scans `repo.my` for each of `siblings` (each a `(id, path)` pair) —
/// present or not, every sibling gets a node so the graph shows the whole
/// ecosystem's adoption state, not just the repos that already comply.
/// Сканує `repo.my` для кожного сусіда — присутній чи ні, кожен сусід
/// отримує вузол, щоб граф показував стан прийняття конвенції по всій
/// екосистемі, а не лише репо, що вже відповідають.
pub fn scan(siblings: &[(&str, &Path)]) -> Vec<RepoNode> {
    siblings
        .iter()
        .map(|(id, path)| {
            fs::read_to_string(path.join("repo.my"))
                .ok()
                .and_then(|raw| parse_repo_node(id, &raw))
                .unwrap_or_else(|| RepoNode::missing(id))
        })
        .collect()
}

/// Derives coarse capability-overlap edges: for every (importer, importee)
/// pair where importer's `imports` and importee's `exports` share a name.
/// O(n² × m) over a handful of repos and a dozen capability names — fine.
/// Виводить грубі ребра перетину capability: для кожної пари
/// (імпортер, імпортований), де `imports` імпортера і `exports`
/// імпортованого мають спільну назву.
pub fn derive_edges(nodes: &[RepoNode]) -> Vec<RepoEdge> {
    let mut edges = Vec::new();
    for importer in nodes {
        for capability in &importer.imports {
            for exporter in nodes {
                if exporter.id != importer.id && exporter.exports.contains(capability) {
                    edges.push(RepoEdge {
                        from: importer.id.clone(),
                        to: exporter.id.clone(),
                        via_capability: capability.clone(),
                    });
                }
            }
        }
    }
    edges
}
