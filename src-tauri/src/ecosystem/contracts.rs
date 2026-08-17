use my_lisp::{parse, Expr, ExprKind};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
pub struct Version2 {
    pub major: i64,
    pub minor: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageContract {
    pub version: Version2,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsaContract {
    pub version: Version2,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmlCompatibility {
    pub compiler_version: (i64, i64, i64),
    pub language_contract: Version2,
    pub language_sha: Option<String>,
    pub isa_contract: Version2,
    pub isa_sha: Option<String>,
}

pub(super) fn as_list(expr: &Expr) -> Option<&[Expr]> {
    match &expr.kind {
        ExprKind::List(items) => Some(items),
        _ => None,
    }
}

pub(super) fn symbol_name(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Symbol(name) => Some(name),
        _ => None,
    }
}

pub(super) fn number(expr: &Expr) -> Option<i64> {
    match &expr.kind {
        ExprKind::Number(value) => Some(value.round() as i64),
        _ => None,
    }
}

pub(super) fn string_value(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::String(value) => Some(value.to_string()),
        _ => None,
    }
}

/// Looks up `(key . value)` in a contract alist as my-lisp's own parser
/// tokenizes it — three flat items (`Symbol(key)`, `Symbol(".")`, value),
/// since this parser doesn't build real cons pairs for reader dots.
/// Шукає `(key . value)` в alist-контракті так, як його токенізує
/// власний парсер my-lisp — три плоскі елементи, оскільки цей парсер не
/// будує справжні dotted-пари для крапки читача.
pub(super) fn assoc<'a>(items: &'a [Expr], key: &str) -> Option<&'a Expr> {
    items.iter().find_map(|item| {
        let pair = as_list(item)?;
        if pair.len() == 3 && symbol_name(&pair[0])? == key && symbol_name(&pair[1])? == "." {
            Some(&pair[2])
        } else {
            None
        }
    })
}

fn version2(expr: &Expr) -> Option<Version2> {
    let items = as_list(expr)?;
    Some(Version2 {
        major: number(items.first()?)?,
        minor: number(items.get(1)?)?,
    })
}

/// Parses a contract file's single top-level form as my-lisp source and
/// returns its alist entries, using the language's own reader instead of
/// hand-rolled string scanning.
/// Парсить єдину верхньорівневу форму файлу контракту як вихідний код
/// my-lisp і повертає її alist-записи, використовуючи власний reader
/// мови замість саморобного сканування рядків.
pub(super) fn parse_alist(source: &str) -> Option<Vec<Expr>> {
    let forms = parse(source).ok()?;
    let top = forms.into_iter().next()?;
    as_list(&top).map(|items| items.to_vec())
}

/// Reads `language-contract.my`: `((major . 1) (minor . 0) ...)`.
/// Читає `language-contract.my`: `((major . 1) (minor . 0) ...)`.
pub fn read_language_contract(repo: &Path) -> Option<LanguageContract> {
    let raw = fs::read_to_string(repo.join("language-contract.my")).ok()?;
    let items = parse_alist(&raw)?;
    Some(LanguageContract {
        version: Version2 {
            major: number(assoc(&items, "major")?)?,
            minor: number(assoc(&items, "minor")?)?,
        },
    })
}

/// Reads `isa-contract.my`: `(version . (0 2))`.
/// Читає `isa-contract.my`: `(version . (0 2))`.
pub fn read_isa_contract(repo: &Path) -> Option<IsaContract> {
    let raw = fs::read_to_string(repo.join("isa-contract.my")).ok()?;
    let items = parse_alist(&raw)?;
    Some(IsaContract {
        version: version2(assoc(&items, "version")?)?,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmlStatus {
    pub tier1_skips_remaining: i64,
    pub ci_status: String,
    pub equal_status: String,
    pub defmacro_status: String,
}

/// Reads `ecosystem-status.my`'s `cml` entry from the `repositories` alist:
/// the hand-refreshed snapshot of cml's Tier-1 fixture progress and CI state.
/// Читає запис `cml` з alist `repositories` у `ecosystem-status.my`: знімок
/// прогресу Tier-1 фікстур cml і стану CI, що оновлюється вручну.
pub fn read_cml_status(repo: &Path) -> Option<CmlStatus> {
    let raw = fs::read_to_string(repo.join("ecosystem-status.my")).ok()?;
    let items = parse_alist(&raw)?;

    let repositories = as_list(assoc(&items, "repositories")?)?;
    let cml = as_list(assoc(repositories, "cml")?)?;

    Some(CmlStatus {
        tier1_skips_remaining: number(assoc(cml, "tier-1-skips-remaining")?)?,
        ci_status: symbol_name(assoc(cml, "ci-status")?)?.to_string(),
        equal_status: symbol_name(assoc(cml, "equal-status")?)?.to_string(),
        defmacro_status: symbol_name(assoc(cml, "defmacro-status")?)?.to_string(),
    })
}

/// Reads `compatibility.my`, the CML boundary contract: compiler version plus the
/// language-contract and ISA versions it was last tested against.
/// Читає `compatibility.my`, контракт межі CML: версію компілятора та версії
/// language-contract і ISA, з якими він востаннє тестувався.
pub fn read_cml_compatibility(repo: &Path) -> Option<CmlCompatibility> {
    let raw = fs::read_to_string(repo.join("compatibility.my")).ok()?;
    let items = parse_alist(&raw)?;

    let compiler_items = as_list(assoc(&items, "compiler-version")?)?;
    let compiler_version = (
        number(compiler_items.first()?)?,
        number(compiler_items.get(1)?)?,
        number(compiler_items.get(2)?)?,
    );

    let language_items = as_list(assoc(&items, "language")?)?;
    let language_contract = version2(assoc(language_items, "contract")?)?;
    let language_sha = assoc(language_items, "tested-sha").and_then(string_value);

    let target_items = as_list(assoc(&items, "target")?)?;
    let isa_contract = version2(assoc(target_items, "isa")?)?;
    let isa_sha = assoc(target_items, "tested-sha").and_then(string_value);

    Some(CmlCompatibility {
        compiler_version,
        language_contract,
        language_sha,
        isa_contract,
        isa_sha,
    })
}
