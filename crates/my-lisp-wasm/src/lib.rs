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

use my_lisp::{eval_program, parse, Session};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Result shape identical to the Tauri LispEvaluation struct.
/// Форма результату ідентична структурі Tauri LispEvaluation.
/// Ergebnisstruktur identisch mit der Tauri-LispEvaluation-Struktur.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Evaluation {
    value: String,
    output: Vec<String>,
    ast: String,
    engine: &'static str,
}

/// Evaluates a my-lisp program and returns `{ value, output, ast, engine }`.
/// On error returns a JS exception (caught by the `.catch` handler in CLJS).
///
/// Обчислює програму my-lisp і повертає `{ value, output, ast, engine }`.
/// При помилці кидає JS-виняток (перехоплюється обробником `.catch` у CLJS).
///
/// Wertet ein my-lisp-Programm aus und gibt `{ value, output, ast, engine }` zurück.
/// Im Fehlerfall wird eine JS-Ausnahme geworfen (im CLJS-.catch-Handler abgefangen).
#[wasm_bindgen]
pub fn evaluate(source: &str) -> Result<JsValue, JsValue> {
    let forms = parse(source).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = eval_program(source, &mut session)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let evaluation = Evaluation {
        value: result.value.to_string(),
        output: result.output,
        ast: format!("{forms:#?}"),
        engine: "my-lisp · WASM",
    };

    serde_wasm_bindgen::to_value(&evaluation).map_err(|e| JsValue::from_str(&e.to_string()))
}
