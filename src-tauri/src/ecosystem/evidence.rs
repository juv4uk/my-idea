use super::contracts::{as_list, assoc, parse_alist, string_value, symbol_name};
use my_lisp::Expr;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRecord {
    pub fixture: String,
    pub requirement: String,
    pub implementation: String,
    pub commit: String,
    pub runner: String,
    pub expected: String,
    pub actual: String,
    pub result: String,
    pub timestamp: String,
    pub note: Option<String>,
    pub environment: Option<EvidenceEnvironment>,
}

/// Optional per `evidence/README.md`'s "Optional `environment` field":
/// pins the toolchain state (Guix) a run was produced under, on top of
/// `commit` pinning the code state. Absent on evidence files written before
/// this was added — that's expected, not a parse failure.
/// Опційне поле `environment`: фіксує стан toolchain (Guix), на додачу
/// до `commit`, що фіксує стан коду. Відсутнє в старіших evidence-файлах
/// — це очікувано, не помилка парсингу.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceEnvironment {
    pub guix_revision: Option<String>,
    pub channels: Option<String>,
    pub manifest: Option<String>,
}

/// Finds `(tag (key . value) ...)` among `items` — a nested tagged list,
/// as opposed to `assoc`'s flat `(key . value)` pairs. `environment` is
/// shaped this way, same as the outer `(evidence ...)` wrapper itself.
/// Шукає `(tag (key . value) ...)` серед `items` — вкладений тегований
/// список, на відміну від пласких пар `assoc`.
fn tagged_list<'a>(items: &'a [Expr], tag: &str) -> Option<&'a [Expr]> {
    items.iter().find_map(|item| {
        let sub = as_list(item)?;
        (symbol_name(sub.first()?)? == tag).then_some(sub)
    })
}

/// Parses one `evidence/<requirement>/<implementation>/<sha>.my` file, per the
/// schema at `evidence/README.md`: `(evidence (key . value) ...)`. Leading
/// `;` comment lines (fpga-lisp's convention) are handled by my-lisp's own
/// reader, same as every other contract file this module parses.
/// Парсить один файл `evidence/<requirement>/<implementation>/<sha>.my` за
/// схемою з `evidence/README.md`. Провідні `;`-коментарі (конвенція
/// fpga-lisp) обробляє власний reader my-lisp, як і решта контрактів.
fn parse_record(raw: &str) -> Option<EvidenceRecord> {
    let items = parse_alist(raw)?;
    Some(EvidenceRecord {
        fixture: string_value(assoc(&items, "fixture")?)?,
        requirement: symbol_name(assoc(&items, "requirement")?)?.to_string(),
        implementation: symbol_name(assoc(&items, "implementation")?)?.to_string(),
        commit: string_value(assoc(&items, "commit")?)?,
        runner: assoc(&items, "runner")
            .and_then(|e| string_value(e).or_else(|| symbol_name(e).map(str::to_string)))?,
        expected: string_value(assoc(&items, "expected")?)?,
        actual: string_value(assoc(&items, "actual")?)?,
        result: symbol_name(assoc(&items, "result")?)?.to_string(),
        timestamp: string_value(assoc(&items, "timestamp")?)?,
        note: assoc(&items, "note").and_then(string_value),
        environment: tagged_list(&items, "environment").map(|env| EvidenceEnvironment {
            guix_revision: assoc(env, "guix-revision").and_then(string_value),
            channels: assoc(env, "channels").and_then(string_value),
            manifest: assoc(env, "manifest").and_then(string_value),
        }),
    })
}

/// Walks `<repo>/evidence/<requirement>/<implementation>/*.my` and parses
/// every record found. Missing `evidence/` (not every repo need have one
/// yet) is not an error — just no records.
/// Обходить `<repo>/evidence/<requirement>/<implementation>/*.my` і парсить
/// усі знайдені записи. Відсутня `evidence/` — не помилка, просто немає
/// записів.
pub fn scan(repo: &Path) -> Vec<EvidenceRecord> {
    let evidence_dir = repo.join("evidence");
    let Ok(requirement_dirs) = fs::read_dir(&evidence_dir) else {
        return Vec::new();
    };

    let mut records = Vec::new();
    for requirement_entry in requirement_dirs.flatten() {
        let Ok(implementation_dirs) = fs::read_dir(requirement_entry.path()) else {
            continue;
        };
        for implementation_entry in implementation_dirs.flatten() {
            let Ok(files) = fs::read_dir(implementation_entry.path()) else {
                continue;
            };
            for file in files.flatten() {
                let path = file.path();
                if path.extension().and_then(|e| e.to_str()) != Some("my") {
                    continue;
                }
                if let Ok(raw) = fs::read_to_string(&path) {
                    if let Some(record) = parse_record(&raw) {
                        records.push(record);
                    }
                }
            }
        }
    }
    records
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementRow {
    pub requirement: String,
    /// implementation name -> latest record for this requirement
    pub by_implementation: BTreeMap<String, EvidenceRecord>,
}

/// Merges evidence from all repos into one requirement-keyed matrix, keeping
/// only the most recent record per (requirement, implementation) — evidence
/// files accumulate one per commit, so old ones stay on disk as history but
/// the matrix should reflect current state. "Most recent" is by timestamp,
/// falling back to file order (README's `timestamp` is date-only, so same-day
/// records tie-break on whichever was scanned last).
/// Зливає evidence з усіх репо в одну requirement-матрицю, лишаючи тільки
/// найновіший запис на пару (requirement, implementation).
pub fn matrix(records: Vec<EvidenceRecord>) -> Vec<RequirementRow> {
    let mut rows: BTreeMap<String, BTreeMap<String, EvidenceRecord>> = BTreeMap::new();
    for record in records {
        let by_impl = rows.entry(record.requirement.clone()).or_default();
        let is_newer = by_impl
            .get(&record.implementation)
            .map(|existing| record.timestamp >= existing.timestamp)
            .unwrap_or(true);
        if is_newer {
            by_impl.insert(record.implementation.clone(), record);
        }
    }
    rows.into_iter()
        .map(|(requirement, by_implementation)| RequirementRow {
            requirement,
            by_implementation,
        })
        .collect()
}
