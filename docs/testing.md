# my-idea test results · Результати тестів my-idea · my-idea-Testergebnisse

## English

The project has two independent test layers: the Rust crates under `crates/` (run with `cargo test`), and the Node/Playwright suite that exercises the ClojureScript frontend, the WASM engine, and the standalone web artifacts (run with `bun run test`). There is no cross-suite coverage tool configured yet; this table is the current source of truth and should be refreshed whenever a suite gains or loses tests.

### Rust crates — `cargo test`

| Crate | Suite | Tests | Covers |
|---|---|---:|---|
| `my-lisp` | unit tests (`src/parser.rs`, `src/environment.rs`, `src/eval/mod.rs`) | 24 | reader/parser edge cases, lexical-scope isolation, single-pass evaluation, macro expansion |
| `my-lisp` | `tests/mccarthy.rs` | 12 | the seven McCarthy primitives, exact/inexact arithmetic, lambda semantics, structured errors |
| `my-lisp` | `tests/stack_safety.rs` | 4 | tail recursion and deep list clone/drop use constant Rust stack |
| `my-lisp-cli` | `tests/cli.rs` | 8 | the compiled binary end-to-end: `--version`/`--help`, file execution, parse/eval error exit codes, missing-file handling, `lib/core.my` preloading |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | literate-Markdown source-offset mapping |
| **Rust total** | | **52** | |

```powershell
cargo test --manifest-path crates/my-lisp/Cargo.toml
cargo test --manifest-path crates/my-lisp-cli/Cargo.toml
cargo test --manifest-path crates/my-lisp-literate/Cargo.toml
```

### Web/JS suite — `bun run test` (`node --test tests/*.test.mjs`)

| File | Tests | Covers |
|---|---:|---|
| `tests/conformance.test.mjs` | 19 | implementation-independent fixture cases (`tests/fixtures/conformance.json`) run against the WASM engine directly in Node, plus a 100k-list stack-safety check on the raw WASM adapter |
| `tests/smoke.test.mjs` | 15 | static wiring checks (trilingual UI, PWA manifest/service worker, Tauri commands, WASM/CLJS bindings, release workflow asset names) plus a Playwright check that `my-idea-web.html` doesn't stack-overflow on a 100k-element list |
| `tests/my-lisp-cli-web.test.mjs` | 5 | Playwright end-to-end checks on `public/my-lisp-cli-web.html`: plain arithmetic, definitions persisting across REPL lines, `lib/core.my` preloading, exact rational arithmetic, and an error not corrupting the session |
| **Web/JS total** | **39** | |

`bun run test` additionally runs `shadow-cljs compile test`, a ClojureScript test-compilation step that currently contains 0 assertions (reserved for future CLJS-level unit tests; the Node suite above is where actual coverage lives today).

```powershell
bun run test
```

### Grand total

**91 automated tests** (52 Rust + 39 Web/JS) across the project, last recorded run: 2026-08-08, Windows x86_64 — all passing, 0 failed, 0 ignored.

## Українська

Проєкт має два незалежні шари тестів: Rust-крейти в `crates/` (запуск через `cargo test`) і Node/Playwright-набір, що перевіряє ClojureScript-фронтенд, WASM-рушій і standalone web-артефакти (запуск через `bun run test`). Інструмент для наскрізного покриття між шарами поки не налаштовано; ця таблиця є поточним джерелом правди й має оновлюватися щоразу, коли набір отримує або втрачає тести.

### Rust-крейти — `cargo test`

| Крейт | Набір | Тестів | Покриває |
|---|---|---:|---|
| `my-lisp` | unit-тести (`src/parser.rs`, `src/environment.rs`, `src/eval/mod.rs`) | 24 | межові випадки reader/parser, ізоляцію лексичного скоупу, однопрохідне обчислення, розкриття макросів |
| `my-lisp` | `tests/mccarthy.rs` | 12 | сім примітивів Маккарті, точну/неточну арифметику, семантику lambda, структуровані помилки |
| `my-lisp` | `tests/stack_safety.rs` | 4 | хвостову рекурсію та clone/drop глибоких списків зі сталим Rust-стеком |
| `my-lisp-cli` | `tests/cli.rs` | 8 | скомпільований бінарник наскрізно: `--version`/`--help`, виконання файлу, коди виходу при помилках парсингу/обчислення, відсутній файл, попереднє завантаження `lib/core.my` |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | зіставлення зміщень початкового коду literate-Markdown |
| **Разом Rust** | | **52** | |

```powershell
cargo test --manifest-path crates/my-lisp/Cargo.toml
cargo test --manifest-path crates/my-lisp-cli/Cargo.toml
cargo test --manifest-path crates/my-lisp-literate/Cargo.toml
```

### Web/JS-набір — `bun run test` (`node --test tests/*.test.mjs`)

| Файл | Тестів | Покриває |
|---|---:|---|
| `tests/conformance.test.mjs` | 19 | незалежні від реалізації fixture-кейси (`tests/fixtures/conformance.json`), що запускаються проти WASM-рушія напряму в Node, плюс перевірка stack-safety на 100k-списку для сирого WASM-адаптера |
| `tests/smoke.test.mjs` | 15 | статичні перевірки підключення (трилінгвальний UI, PWA manifest/service worker, Tauri-команди, WASM/CLJS-прив'язки, назви release-asset у workflow) плюс Playwright-перевірка, що `my-idea-web.html` не переповнює стек на 100k-елементному списку |
| `tests/my-lisp-cli-web.test.mjs` | 5 | Playwright end-to-end перевірки `public/my-lisp-cli-web.html`: звичайна арифметика, збереження визначень між рядками REPL, попереднє завантаження `lib/core.my`, точна раціональна арифметика, і що помилка не псує сесію |
| **Разом Web/JS** | **39** | |

`bun run test` додатково запускає `shadow-cljs compile test` — крок компіляції ClojureScript-тестів, що наразі містить 0 тверджень (зарезервовано під майбутні CLJS-unit-тести; реальне покриття сьогодні живе в Node-наборі вище).

```powershell
bun run test
```

### Загальний підсумок

**91 автотест** (52 Rust + 39 Web/JS) у проєкті, останній зафіксований запуск: 2026-08-08, Windows x86_64 — усі проходять, 0 провалів, 0 пропущено.

## Deutsch

Das Projekt hat zwei unabhängige Testebenen: die Rust-Crates unter `crates/` (ausgeführt mit `cargo test`) und die Node/Playwright-Suite, die das ClojureScript-Frontend, die WASM-Engine und die eigenständigen Web-Artefakte prüft (ausgeführt mit `bun run test`). Ein ebenenübergreifendes Coverage-Werkzeug ist noch nicht eingerichtet; diese Tabelle ist die aktuelle Quelle der Wahrheit und sollte aktualisiert werden, sobald eine Suite Tests gewinnt oder verliert.

### Rust-Crates — `cargo test`

| Crate | Suite | Tests | Deckt ab |
|---|---|---:|---|
| `my-lisp` | Unit-Tests (`src/parser.rs`, `src/environment.rs`, `src/eval/mod.rs`) | 24 | Reader-/Parser-Grenzfälle, Isolation des lexikalischen Scopes, Single-Pass-Auswertung, Makro-Expansion |
| `my-lisp` | `tests/mccarthy.rs` | 12 | die sieben McCarthy-Primitive, exakte/inexakte Arithmetik, Lambda-Semantik, strukturierte Fehler |
| `my-lisp` | `tests/stack_safety.rs` | 4 | Tail-Rekursion und Clone/Drop tiefer Listen mit konstantem Rust-Stack |
| `my-lisp-cli` | `tests/cli.rs` | 8 | die kompilierte Binärdatei durchgängig: `--version`/`--help`, Dateiausführung, Exit-Codes bei Parse-/Eval-Fehlern, fehlende Datei, Vorladen von `lib/core.my` |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | Offset-Zuordnung von literate-Markdown-Quellcode |
| **Rust gesamt** | | **52** | |

```powershell
cargo test --manifest-path crates/my-lisp/Cargo.toml
cargo test --manifest-path crates/my-lisp-cli/Cargo.toml
cargo test --manifest-path crates/my-lisp-literate/Cargo.toml
```

### Web/JS-Suite — `bun run test` (`node --test tests/*.test.mjs`)

| Datei | Tests | Deckt ab |
|---|---:|---|
| `tests/conformance.test.mjs` | 19 | implementierungsunabhängige Fixture-Fälle (`tests/fixtures/conformance.json`), direkt gegen die WASM-Engine in Node ausgeführt, plus eine Stack-Safety-Prüfung mit 100k-Liste am rohen WASM-Adapter |
| `tests/smoke.test.mjs` | 15 | statische Verdrahtungsprüfungen (dreisprachige UI, PWA-Manifest/Service-Worker, Tauri-Befehle, WASM/CLJS-Bindungen, Release-Workflow-Asset-Namen) plus eine Playwright-Prüfung, dass `my-idea-web.html` bei einer 100k-Elemente-Liste nicht überläuft |
| `tests/my-lisp-cli-web.test.mjs` | 5 | Playwright-End-to-End-Prüfungen von `public/my-lisp-cli-web.html`: einfache Arithmetik, über REPL-Zeilen persistente Definitionen, Vorladen von `lib/core.my`, exakte rationale Arithmetik und dass ein Fehler die Sitzung nicht beschädigt |
| **Web/JS gesamt** | **39** | |

`bun run test` führt zusätzlich `shadow-cljs compile test` aus, einen ClojureScript-Testkompilierungsschritt, der derzeit 0 Assertions enthält (reserviert für künftige CLJS-Unit-Tests; die tatsächliche Abdeckung liegt heute in der obigen Node-Suite).

```powershell
bun run test
```

### Gesamtsumme

**91 automatisierte Tests** (52 Rust + 39 Web/JS) im Projekt, letzter erfasster Lauf: 08.08.2026, Windows x86_64 — alle bestanden, 0 fehlgeschlagen, 0 übersprungen.
