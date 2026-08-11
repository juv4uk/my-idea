use my_lisp::{parse, Expr, ExprKind};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
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

fn as_list(expr: &Expr) -> Option<&[Expr]> {
    match &expr.kind {
        ExprKind::List(items) => Some(items),
        _ => None,
    }
}

fn symbol_name(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Symbol(name) => Some(name),
        _ => None,
    }
}

/// Accepts only values that are exact integers in `i64` range — a contract
/// version field like `(major . 0.5)` or one beyond `i64::MAX` is malformed,
/// not something to silently round or saturate.
fn number(expr: &Expr) -> Option<i64> {
    match &expr.kind {
        ExprKind::Number(value) if value.fract() == 0.0 && value.is_finite() => {
            if *value < i64::MIN as f64 || *value > i64::MAX as f64 {
                None
            } else {
                Some(*value as i64)
            }
        }
        _ => None,
    }
}

fn string_value(expr: &Expr) -> Option<String> {
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
fn assoc<'a>(items: &'a [Expr], key: &str) -> Option<&'a Expr> {
    items.iter().find_map(|item| {
        let pair = as_list(item)?;
        if pair.len() == 3 && symbol_name(&pair[0])? == key && symbol_name(&pair[1])? == "." {
            Some(&pair[2])
        } else {
            None
        }
    })
}

/// Requires exactly two components — `(1 0 2)` or `(1)` is a malformed
/// version, not one to be read leniently and have its extras ignored.
fn version2(expr: &Expr) -> Option<Version2> {
    let items = as_list(expr)?;
    if items.len() != 2 {
        return None;
    }
    Some(Version2 {
        major: number(&items[0])?,
        minor: number(&items[1])?,
    })
}

/// Parses a contract file's single top-level form as my-lisp source and
/// returns its alist entries, using the language's own reader instead of
/// hand-rolled string scanning. A file with more than one top-level form
/// is malformed, not "read the first one and ignore the rest" — matches
/// AGENT_MEMORY.md's "fail on fixture parse loss instead of silently
/// continuing".
/// Парсить єдину верхньорівневу форму файлу контракту як вихідний код
/// my-lisp і повертає її alist-записи, використовуючи власний reader
/// мови замість саморобного сканування рядків. Файл з більш ніж однією
/// верхньорівневою формою вважається неправильним.
fn parse_alist(source: &str) -> Option<Vec<Expr>> {
    let mut forms = parse(source).ok()?;
    if forms.len() != 1 {
        return None;
    }
    as_list(&forms.remove(0)).map(|items| items.to_vec())
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

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Fixture {
    pub index: usize,
    pub expr: String,
    pub expected: Option<String>,
    pub error: Option<String>,
    pub tier: Option<i64>,
    pub axioms: Vec<String>,
    pub role: Option<String>,
    pub note: Option<String>,
}

fn symbol_list(expr: &Expr) -> Option<Vec<String>> {
    as_list(expr)?
        .iter()
        .map(symbol_name)
        .map(|name| name.map(str::to_string))
        .collect()
}

/// Reads the canonical, ordered fixture inventory from my-lisp. The inventory
/// deliberately contains contract facts and classification tags only; execution
/// results and their evidence are added by the later pipeline stage.
pub fn read_fixture_inventory(repo: &Path) -> Vec<Fixture> {
    let Ok(raw) = fs::read_to_string(repo.join("tests/fixtures/conformance.my")) else {
        return Vec::new();
    };
    let Ok(forms) = parse(&raw) else {
        return Vec::new();
    };

    forms
        .iter()
        .enumerate()
        .filter_map(|(offset, form)| {
            let items = as_list(form)?;
            let expected = assoc(items, "expected").and_then(string_value);
            let error = assoc(items, "error").and_then(string_value);
            if expected.is_some() == error.is_some() {
                return None;
            }
            Some(Fixture {
                index: offset + 1,
                expr: string_value(assoc(items, "expr")?)?,
                expected,
                error,
                tier: assoc(items, "tier").and_then(number),
                axioms: assoc(items, "axioms")
                    .and_then(symbol_list)
                    .unwrap_or_default(),
                role: assoc(items, "role").and_then(string_value),
                note: assoc(items, "note").and_then(string_value),
            })
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Removes its directory on drop so parallel `cargo test` runs never see
    /// each other's fixture files and %TEMP% doesn't accumulate test debris.
    struct TempRepo(PathBuf);

    impl std::ops::Deref for TempRepo {
        type Target = Path;
        fn deref(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn temp_repo() -> TempRepo {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "my-idea-contracts-test-{}-{n}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        TempRepo(dir)
    }

    fn write_temp(name: &str, contents: &str) -> TempRepo {
        let repo = temp_repo();
        let path = repo.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, contents).unwrap();
        repo
    }

    #[test]
    fn reads_language_contract_major_minor() {
        let repo = write_temp(
            "language-contract.my",
            r#"((major . 1) (minor . 0)
 (note . "bare integer literals are exact")
 (covers . (G1 G2)))"#,
        );
        let contract = read_language_contract(&repo).expect("should parse");
        assert_eq!(contract.version.major, 1);
        assert_eq!(contract.version.minor, 0);
    }

    #[test]
    fn reads_isa_contract_version() {
        let repo = write_temp(
            "isa-contract.my",
            r#"((kind . isa-contract) (version . (0 2)) (notes . nil))"#,
        );
        let contract = read_isa_contract(&repo).expect("should parse");
        assert_eq!(contract.version.major, 0);
        assert_eq!(contract.version.minor, 2);
    }

    #[test]
    fn reads_cml_compatibility_versions_and_shas() {
        let repo = write_temp(
            "compatibility.my",
            r#"((kind . cml-compatibility)
 (compiler-version . (0 1 0))
 (language . ((repository . juv4uk/my-lisp)
              (contract . (1 0))
              (tested-sha . "ed10151")))
 (target . ((repository . juv4uk/fpga-lisp)
            (isa . (0 2))
            (tested-sha . "01bb01a"))))"#,
        );
        let compat = read_cml_compatibility(&repo).expect("should parse");
        assert_eq!(compat.compiler_version, (0, 1, 0));
        assert_eq!(compat.language_contract, Version2 { major: 1, minor: 0 });
        assert_eq!(compat.language_sha.as_deref(), Some("ed10151"));
        assert_eq!(compat.isa_contract, Version2 { major: 0, minor: 2 });
        assert_eq!(compat.isa_sha.as_deref(), Some("01bb01a"));
    }

    #[test]
    fn reads_cml_status_from_ecosystem_status() {
        let repo = write_temp(
            "ecosystem-status.my",
            r#"((kind . ecosystem-status)
 (as-of . "2026-08-11")
 (repositories .
  ((my-lisp . ((role . semantic-source-of-truth)))
   (cml .
    ((role . aot-compiler)
     (tier-1-skips-remaining . 0)
     (ci-status . green)
     (equal-status . merged-machine-verified)
     (defmacro-status . merged-machine-verified))))))"#,
        );
        let status = read_cml_status(&repo).expect("should parse");
        assert_eq!(status.tier1_skips_remaining, 0);
        assert_eq!(status.ci_status, "green");
        assert_eq!(status.equal_status, "merged-machine-verified");
        assert_eq!(status.defmacro_status, "merged-machine-verified");
    }

    #[test]
    fn missing_file_returns_none() {
        let repo = write_temp("unrelated.my", "()");
        assert!(read_language_contract(&repo).is_none());
    }

    #[test]
    fn missing_file_does_not_see_sibling_files() {
        // Regression test for the parallel-test race OpenCode flagged: every
        // temp_repo() call now gets its own directory, so a test that only
        // wrote an unrelated file can never observe another test's contract.
        let repo = write_temp("language-contract.my", "((major . 1) (minor . 0))");
        let other = write_temp("unrelated.my", "()");
        assert!(read_language_contract(&other).is_none());
        assert!(read_language_contract(&repo).is_some());
    }

    #[test]
    fn rejects_multiple_top_level_forms() {
        let repo = write_temp(
            "language-contract.my",
            "((major . 1) (minor . 0)) ((major . 2) (minor . 0))",
        );
        assert!(read_language_contract(&repo).is_none());
    }

    #[test]
    fn rejects_non_integer_version_components() {
        let repo = write_temp("language-contract.my", "((major . 0.5) (minor . 0))");
        assert!(read_language_contract(&repo).is_none());

        let repo = write_temp("isa-contract.my", "((version . (1.5 0)))");
        assert!(read_isa_contract(&repo).is_none());
    }

    #[test]
    fn rejects_overflowing_numbers() {
        let repo = write_temp(
            "language-contract.my",
            "((major . 100000000000000000000) (minor . 0))",
        );
        assert!(read_language_contract(&repo).is_none());
    }

    #[test]
    fn version_requires_exactly_two_components() {
        let repo = write_temp("isa-contract.my", "((version . (0 2 1)))");
        assert!(read_isa_contract(&repo).is_none());

        let repo = write_temp("isa-contract.my", "((version . (0)))");
        assert!(read_isa_contract(&repo).is_none());
    }

    #[test]
    fn empty_and_comment_only_files_are_none() {
        let repo = write_temp("language-contract.my", "; just a comment\n");
        assert!(read_language_contract(&repo).is_none());
    }

    #[test]
    fn reads_ordered_fixture_inventory_with_success_and_error_cases() {
        let repo = write_temp(
            "tests/fixtures/conformance.my",
            r#"; canonical fixtures
((expr . "(quote radio)") (expected . "radio") (tier . 1) (axioms . (G3)) (role . "constitutive"))
((expr . "(car 5)") (error . "Type") (tier . 1) (axioms . (S2)) (note . "type evidence"))"#,
        );
        let fixtures = read_fixture_inventory(&repo);
        assert_eq!(fixtures.len(), 2);
        assert_eq!(fixtures[0].index, 1);
        assert_eq!(fixtures[0].expr, "(quote radio)");
        assert_eq!(fixtures[0].expected.as_deref(), Some("radio"));
        assert_eq!(fixtures[0].axioms, vec!["G3"]);
        assert_eq!(fixtures[0].role.as_deref(), Some("constitutive"));
        assert_eq!(fixtures[1].index, 2);
        assert_eq!(fixtures[1].error.as_deref(), Some("Type"));
        assert_eq!(fixtures[1].note.as_deref(), Some("type evidence"));
    }

    #[test]
    fn missing_or_invalid_fixture_inventory_is_empty() {
        let repo = write_temp("unrelated.my", "()");
        assert!(read_fixture_inventory(&repo).is_empty());

        let repo = write_temp("tests/fixtures/conformance.my", "not (");
        assert!(read_fixture_inventory(&repo).is_empty());
    }
}
