//! WebAssembly bindings that expose the canonical my-lisp Rust engine to the browser.
//! WASM-прив'язки, що відкривають канонічний Rust-рушій my-lisp для браузера.
//! WebAssembly-Bindungen, die den kanonischen my-lisp-Rust-Engine dem Browser zugänglich machen.
//!
//! The public API intentionally mirrors the Tauri `evaluate_my_lisp` command so that
//! `core.cljs` can treat both environments with identical result shapes.
//!
//! Публічний API навмисно відображає команду Tauri `evaluate_my_lisp`, щоб
//! `core.cljs` міг обробляти обидва середовища з однаковою формою результату.
//!
//! Die öffentliche API spiegelt bewusst den Tauri-Befehl `evaluate_my_lisp`, sodass
//! `core.cljs` beide Umgebungen mit identischen Ergebnisstrukturen behandeln kann.

use my_lisp::{eval_parsed_expressions, eval_program, parse, Session};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Result shape identical to the Tauri LispEvaluation struct.
/// Форма результату ідентична структурі Tauri LispEvaluation.
/// Ergebnisstruktur identisch mit der Tauri-LispEvaluation-Struktur.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Evaluation {
    value: String,
    output: Vec<String>,
    ast: String,
    engine: &'static str,
}

/// Evaluates a my-lisp program and returns `{ value, output, ast, engine }`.
/// Uses a single-pass parse (`eval_parsed_expressions`) to avoid redundant parsing.
/// On error returns a JS exception (caught by the `.catch` handler in CLJS).
///
/// Обчислює програму my-lisp і повертає `{ value, output, ast, engine }`.
/// Використовує однопрохідний парсинг (`eval_parsed_expressions`), щоб уникнути повторного аналізу.
/// При помилці кидає JS-виняток (перехоплюється обробником `.catch` у CLJS).
///
/// Wertet ein my-lisp-Programm aus und gibt `{ value, output, ast, engine }` zurück.
/// Verwendet Single-Pass-Parsing (`eval_parsed_expressions`), um doppeltes Parsing zu vermeiden.
/// Im Fehlerfall wird eine JS-Ausnahme geworfen (im CLJS-.catch-Handler abgefangen).
#[wasm_bindgen]
pub fn evaluate(source: &str) -> Result<JsValue, JsValue> {
    // EN: Single-pass parse for both AST generation and evaluation.
    // UK: Однопрохідний парсинг для побудови AST та обчислення.
    // DE: Single-Pass-Parsing sowohl für AST-Generierung als auch Auswertung.
    let forms = parse(source).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = eval_parsed_expressions(&forms, &mut session)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let evaluation = Evaluation {
        value: result.value.to_string(),
        output: result.output,
        ast: format!("{forms:#?}"),
        engine: "my-lisp · WASM",
    };

    serde_wasm_bindgen::to_value(&evaluation).map_err(|e| JsValue::from_str(&e.to_string()))
}

// EN: Maintainers: If the body of evaluate() is modified above, ensure this native test stays in sync.
// UK: Розробникам: Якщо тіло evaluate() змінюється вище, зберігайте цей нативний тест у синхроні з ним.
// DE: Entwickler: Wenn der Rumpf von evaluate() oben geändert wird, halten Sie diesen nativen Test synchron.
#[cfg(test)]
mod native_wasm_crate_tests {
    use super::*;

    #[test]
    fn wasm_crate_single_pass_produces_exact_evaluation_struct() {
        let forms = parse("(cons (second '(radio antenna)) (cons (/ 1 3) '()))")
            .expect("parse should succeed");
        let mut session = Session::default();
        eval_program(include_str!("../../../lib/core.my"), &mut session)
            .expect("bootstrap should succeed");
        let result = eval_parsed_expressions(&forms, &mut session)
            .expect("eval_parsed_expressions should succeed");

        let evaluation = Evaluation {
            value: result.value.to_string(),
            output: result.output,
            ast: format!("{forms:#?}"),
            engine: "my-lisp · WASM",
        };

        assert_eq!(evaluation.value, "(antenna 1/3)");
        assert_eq!(evaluation.engine, "my-lisp · WASM");
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_adapter_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn wasm_adapter_single_pass_preserves_exact_rationals_and_serde_boundary() {
        let js_value = evaluate("(cons (second '(radio antenna)) (cons (/ 1 3) '()))")
            .expect("WASM evaluation should succeed for exact values");
        let eval: Evaluation = serde_wasm_bindgen::from_value(js_value)
            .expect("should deserialize Evaluation struct from JsValue");
        assert_eq!(eval.value, "(antenna 1/3)");
        assert_eq!(eval.engine, "my-lisp · WASM");
    }
}
