# my-idea test results · Результати тестів my-idea · my-idea-Testergebnisse

## English

The project has two independent test layers: the Rust crates under `external/my-lisp/crates/` plus `src-tauri` (run with `cargo test`), and the Node/Playwright suite that exercises the ClojureScript frontend, the WASM engine, and the standalone web artifacts (run with `npm test`). There is no cross-suite coverage tool configured yet; this table is the current source of truth and should be refreshed whenever a suite gains or loses tests.

Before the web tests or release build, install dependencies with `npm ci` and build the canonical engine with `npm run wasm`. The latter targets `external/my-lisp/crates/my-lisp-wasm` and writes generated bindings to `public/wasm`; it requires `wasm-pack` and the `wasm32-unknown-unknown` Rust target.

### Rust crates — `cargo test`

| Crate | Suite | Tests | Covers |
|---|---|---:|---|
| `my-lisp` | unit tests (`src/parser.rs`, `src/environment.rs`, `src/eval/mod.rs`) | 35 | reader/parser edge cases, lexical-scope isolation, single-pass evaluation, macro expansion |
| `my-lisp` | `tests/forward.rs` | 26 | `lib/forward.my` — CLIPS-style forward-chaining rule engine |
| `my-lisp` | `tests/knowledge.rs` | 8 | `lib/knowledge.my` knowledge packages over core, unify, reason |
| `my-lisp` | `tests/mccarthy.rs` | 27 | the seven McCarthy primitives, exact/inexact arithmetic, lambda semantics, structured errors |
| `my-lisp` | `tests/meta_eval.rs` | 9 | `lib/meta-eval.my` metacircular evaluator via `(my-eval (read "...") env)` |
| `my-lisp` | `tests/narrate.rs` | 5 | `lib/narrate.my` — the "structure → text" half of the knowledge bridge |
| `my-lisp` | `tests/reason.rs` | 13 | `lib/reason.my` symbolic inference engine (Advice Taker style) |
| `my-lisp` | `tests/stack_safety.rs` | 5 | tail recursion and deep list clone/drop use constant Rust stack |
| `my-lisp` | `tests/understand.rs` | 5 | `lib/understand.my` — the "word list → knowledge clause" half of the bridge |
| `my-lisp` | `tests/unify.rs` | 12 | `lib/unify.my` unification primitive behind backward-chaining reasoning |
| `my-lisp-cli` | `tests/cli.rs` | 10 | the compiled binary end-to-end: `--version`/`--help`, file execution, parse/eval error exit codes, missing-file handling, `lib/core.my` preloading |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | literate-Markdown source-offset mapping |
| `src-tauri` | unit tests (`src/lib.rs`, `src/ecosystem/contracts.rs`) | 8 | native evaluator, workspace path boundaries, contract readers, ordered fixture inventory |
| **Rust total** | | **167** | |

```powershell
cargo test --manifest-path external/my-lisp/crates/my-lisp/Cargo.toml
cargo test --manifest-path external/my-lisp/crates/my-lisp-cli/Cargo.toml
cargo test --manifest-path external/my-lisp/crates/my-lisp-literate/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

### Web/JS suite — `npm test` (`node --test tests/*.test.mjs`)

| File | Tests | Covers |
|---|---:|---|
| `tests/conformance.test.mjs` | 19 | implementation-independent fixture cases (`tests/fixtures/conformance.json`) run against the WASM engine directly in Node, plus a 100k-list stack-safety check on the raw WASM adapter |
| `tests/smoke.test.mjs` | 15 | static wiring checks (including the vendored WASM build path and the three-card System Observatory UI), plus a Playwright check that `my-idea-web.html` doesn't stack-overflow on a 100k-element list |
| **Web/JS total** | **34** | |

`npm test` additionally runs `shadow-cljs compile test`, a ClojureScript test-compilation step that currently contains 0 assertions (reserved for future CLJS-level unit tests; the Node suite above is where actual coverage lives today).

```powershell
npm test
```

### Grand total

**201 automated tests** (167 Rust + 34 Web/JS) are declared across the project. Verification on 2026-08-11 is recorded below; totals should only be described as passing when every listed command completes in the current environment.

### Latest verification (2026-08-11, Windows x86_64)

- `npm ci`: passed; dependencies installed from `package-lock.json`.
- `npm run wasm`: passed; `wasm-pack` built the vendored engine into `public/wasm`.
- `cargo test` (my-lisp, my-lisp-cli, my-lisp-literate, src-tauri): 167 passed, 0 failed.
- `node --test tests/*.test.mjs`: 34 passed, 0 failed, 0 skipped.

## Українська

Проєкт має два незалежні шари тестів: Rust-крейти в `crates/` (запуск через `cargo test`) і Node/Playwright-набір, що перевіряє ClojureScript-фронтенд, WASM-рушій і standalone web-артефакти (запуск через `npm test`). Інструмент для наскрізного покриття між шарами поки не налаштовано; ця таблиця є поточним джерелом правди й має оновлюватися щоразу, коли набір отримує або втрачає тести.

### Rust-крейти — `cargo test`

| Крейт | Набір | Тестів | Покриває |
|---|---|---:|---|
| `my-lisp` | unit-тести (`src/parser.rs`, `src/environment.rs`, `src/eval/mod.rs`) | 35 | межові випадки reader/parser, ізоляцію лексичного скоупу, однопрохідне обчислення, розкриття макросів |
| `my-lisp` | `tests/forward.rs` | 26 | `lib/forward.my` — forward-chaining рушій правил у стилі CLIPS |
| `my-lisp` | `tests/knowledge.rs` | 8 | knowledge-пакети `lib/knowledge.my` поверх core, unify, reason |
| `my-lisp` | `tests/mccarthy.rs` | 27 | сім примітивів Маккарті, точну/неточну арифметику, семантику lambda, структуровані помилки |
| `my-lisp` | `tests/meta_eval.rs` | 9 | метациркулярний evaluator `lib/meta-eval.my` через `(my-eval (read "...") env)` |
| `my-lisp` | `tests/narrate.rs` | 5 | `lib/narrate.my` — половина «структура → текст» містка знань |
| `my-lisp` | `tests/reason.rs` | 13 | `lib/reason.my` — символьний рушій висновків (у дусі Advice Taker) |
| `my-lisp` | `tests/stack_safety.rs` | 5 | хвостову рекурсію та clone/drop глибоких списків зі сталим Rust-стеком |
| `my-lisp` | `tests/understand.rs` | 5 | `lib/understand.my` — половина «список слів → клауза знання» містка |
| `my-lisp` | `tests/unify.rs` | 12 | `lib/unify.my` — примітив уніфікації для backward-chaining міркувань |
| `my-lisp-cli` | `tests/cli.rs` | 10 | скомпільований бінарник наскрізно: `--version`/`--help`, виконання файлу, коди виходу при помилках парсингу/обчислення, відсутній файл, попереднє завантаження `lib/core.my` |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | зіставлення зміщень початкового коду literate-Markdown |
| `src-tauri` | unit-тести (`src/lib.rs`, `src/ecosystem/contracts.rs`) | 8 | нативний evaluator, межі шляхів workspace, читання контрактів, впорядкований fixture inventory |
| **Разом Rust** | | **167** | |

```powershell
cargo test --manifest-path external/my-lisp/crates/my-lisp/Cargo.toml
cargo test --manifest-path external/my-lisp/crates/my-lisp-cli/Cargo.toml
cargo test --manifest-path external/my-lisp/crates/my-lisp-literate/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

### Web/JS-набір — `npm test` (`node --test tests/*.test.mjs`)

| Файл | Тестів | Покриває |
|---|---:|---|
| `tests/conformance.test.mjs` | 19 | незалежні від реалізації fixture-кейси (`tests/fixtures/conformance.json`), що запускаються проти WASM-рушія напряму в Node, плюс перевірка stack-safety на 100k-списку для сирого WASM-адаптера |
| `tests/smoke.test.mjs` | 15 | статичні перевірки підключення (трилінгвальний UI, PWA manifest/service worker, Tauri-команди, WASM/CLJS-прив'язки, назви release-asset у workflow) плюс Playwright-перевірка, що `my-idea-web.html` не переповнює стек на 100k-елементному списку |
| **Разом Web/JS** | **34** | |

`npm test` додатково запускає `shadow-cljs compile test` — крок компіляції ClojureScript-тестів, що наразі містить 0 тверджень (зарезервовано під майбутні CLJS-unit-тести; реальне покриття сьогодні живе в Node-наборі вище).

```powershell
npm test
```

### Загальний підсумок

**201 автотест** (167 Rust + 34 Web/JS) оголошено у проєкті. Перевірено 2026-08-11 на Windows x86_64: `npm ci` і `npm run wasm` проходять; `cargo test` — 167 пройдено, 0 провалів; `node --test tests/*.test.mjs` — 34 пройдено, 0 провалів, 0 пропущено.

## Deutsch

Das Projekt hat zwei unabhängige Testebenen: die Rust-Crates unter `crates/` (ausgeführt mit `cargo test`) und die Node/Playwright-Suite, die das ClojureScript-Frontend, die WASM-Engine und die eigenständigen Web-Artefakte prüft (ausgeführt mit `npm test`). Ein ebenenübergreifendes Coverage-Werkzeug ist noch nicht eingerichtet; diese Tabelle ist die aktuelle Quelle der Wahrheit und sollte aktualisiert werden, sobald eine Suite Tests gewinnt oder verliert.

### Rust-Crates — `cargo test`

| Crate | Suite | Tests | Deckt ab |
|---|---|---:|---|
| `my-lisp` | Unit-Tests (`src/parser.rs`, `src/environment.rs`, `src/eval/mod.rs`) | 35 | Reader-/Parser-Grenzfälle, Isolation des lexikalischen Scopes, Single-Pass-Auswertung, Makro-Expansion |
| `my-lisp` | `tests/forward.rs` | 26 | `lib/forward.my` — CLIPS-artige Forward-Chaining-Regel-Engine |
| `my-lisp` | `tests/knowledge.rs` | 8 | `lib/knowledge.my`-Wissenspakete auf core, unify, reason |
| `my-lisp` | `tests/mccarthy.rs` | 27 | die sieben McCarthy-Primitive, exakte/inexakte Arithmetik, Lambda-Semantik, strukturierte Fehler |
| `my-lisp` | `tests/meta_eval.rs` | 9 | metazirkulärer Evaluator `lib/meta-eval.my` via `(my-eval (read "...") env)` |
| `my-lisp` | `tests/narrate.rs` | 5 | `lib/narrate.my` — die „Struktur → Text“-Hälfte der Wissensbrücke |
| `my-lisp` | `tests/reason.rs` | 13 | `lib/reason.my` — symbolische Inferenz-Engine (Advice-Taker-Stil) |
| `my-lisp` | `tests/stack_safety.rs` | 5 | Tail-Rekursion und Clone/Drop tiefer Listen mit konstantem Rust-Stack |
| `my-lisp` | `tests/understand.rs` | 5 | `lib/understand.my` — die „Wortliste → Wissensklausel“-Hälfte der Brücke |
| `my-lisp` | `tests/unify.rs` | 12 | `lib/unify.my`-Unifikationsprimitive hinter Backward-Chaining-Reasoning |
| `my-lisp-cli` | `tests/cli.rs` | 10 | die kompilierte Binärdatei durchgängig: `--version`/`--help`, Dateiausführung, Exit-Codes bei Parse-/Eval-Fehlern, fehlende Datei, Vorladen von `lib/core.my` |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | Offset-Zuordnung von literate-Markdown-Quellcode |
| `src-tauri` | Unit-Tests (`src/lib.rs`, `src/ecosystem/contracts.rs`) | 8 | nativer Evaluator, Workspace-Pfadgrenzen, Contract-Reader, geordnetes Fixture-Inventar |
| **Rust gesamt** | | **167** | |

```powershell
cargo test --manifest-path external/my-lisp/crates/my-lisp/Cargo.toml
cargo test --manifest-path external/my-lisp/crates/my-lisp-cli/Cargo.toml
cargo test --manifest-path external/my-lisp/crates/my-lisp-literate/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

### Web/JS-Suite — `npm test` (`node --test tests/*.test.mjs`)

| Datei | Tests | Deckt ab |
|---|---:|---|
| `tests/conformance.test.mjs` | 19 | implementierungsunabhängige Fixture-Fälle (`tests/fixtures/conformance.json`), direkt gegen die WASM-Engine in Node ausgeführt, plus eine Stack-Safety-Prüfung mit 100k-Liste am rohen WASM-Adapter |
| `tests/smoke.test.mjs` | 15 | statische Verdrahtungsprüfungen (dreisprachige UI, PWA-Manifest/Service-Worker, Tauri-Befehle, WASM/CLJS-Bindungen, Release-Workflow-Asset-Namen) plus eine Playwright-Prüfung, dass `my-idea-web.html` bei einer 100k-Elemente-Liste nicht überläuft |
| **Web/JS gesamt** | **34** | |

`npm test` führt zusätzlich `shadow-cljs compile test` aus, einen ClojureScript-Testkompilierungsschritt, der derzeit 0 Assertions enthält (reserviert für künftige CLJS-Unit-Tests; die tatsächliche Abdeckung liegt heute in der obigen Node-Suite).

```powershell
npm test
```

### Gesamtsumme

**201 automatisierte Tests** (167 Rust + 34 Web/JS) sind im Projekt deklariert. Verifiziert am 11.08.2026 unter Windows x86_64: `npm ci` und `npm run wasm` erfolgreich; `cargo test` — 167 bestanden, 0 fehlgeschlagen; `node --test tests/*.test.mjs` — 34 bestanden, 0 fehlgeschlagen, 0 übersprungen.
