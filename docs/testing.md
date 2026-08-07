# my-lisp test results · Результати тестів my-lisp · my-lisp-Testergebnisse

## English

Every Rust crate under `crates/` carries its own automated tests, run with `cargo test --manifest-path crates/<crate>/Cargo.toml`. There is no cross-crate coverage tool configured yet; this table is the current source of truth and should be refreshed whenever a crate gains or loses tests.

| Crate | Suite | Tests | Covers |
|---|---|---:|---|
| `my-lisp` | unit tests (`src/parser.rs`, `src/environment.rs`, `src/eval.rs`) | 24 | reader/parser edge cases, lexical-scope isolation, single-pass evaluation, macro expansion |
| `my-lisp` | `tests/mccarthy.rs` | 12 | the seven McCarthy primitives, exact/inexact arithmetic, lambda semantics, structured errors |
| `my-lisp` | `tests/stack_safety.rs` | 4 | tail recursion and deep list clone/drop use constant Rust stack |
| `my-lisp-cli` | `tests/cli.rs` | 8 | the compiled binary end-to-end: `--version`/`--help`, file execution, parse/eval error exit codes, missing-file handling, `lib/core.my` preloading |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | literate-Markdown source-offset mapping |
| **Total** | | **52** | |

Last recorded run: 2026-08-07, Windows x86_64, debug profile — all 52 tests passed, 0 failed, 0 ignored.

Run everything with:

```powershell
cargo test --manifest-path crates/my-lisp/Cargo.toml
cargo test --manifest-path crates/my-lisp-cli/Cargo.toml
cargo test --manifest-path crates/my-lisp-literate/Cargo.toml
```

## Українська

Кожен Rust-крейт у `crates/` має власні автоматичні тести, які запускаються через `cargo test --manifest-path crates/<crate>/Cargo.toml`. Інструмент для наскрізного покриття між крейтами поки не налаштовано; ця таблиця є поточним джерелом правди й має оновлюватися щоразу, коли крейт отримує або втрачає тести.

| Крейт | Набір | Тестів | Покриває |
|---|---|---:|---|
| `my-lisp` | unit-тести (`src/parser.rs`, `src/environment.rs`, `src/eval.rs`) | 24 | межові випадки reader/parser, ізоляцію лексичного скоупу, однопрохідне обчислення, розкриття макросів |
| `my-lisp` | `tests/mccarthy.rs` | 12 | сім примітивів Маккарті, точну/неточну арифметику, семантику lambda, структуровані помилки |
| `my-lisp` | `tests/stack_safety.rs` | 4 | хвостову рекурсію та clone/drop глибоких списків зі сталим Rust-стеком |
| `my-lisp-cli` | `tests/cli.rs` | 8 | скомпільований бінарник наскрізно: `--version`/`--help`, виконання файлу, коди виходу при помилках парсингу/обчислення, відсутній файл, попереднє завантаження `lib/core.my` |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | зіставлення зміщень початкового коду literate-Markdown |
| **Разом** | | **52** | |

Останній зафіксований запуск: 2026-08-07, Windows x86_64, debug-профіль — усі 52 тести пройшли, 0 провалів, 0 пропущено.

Запуск усього:

```powershell
cargo test --manifest-path crates/my-lisp/Cargo.toml
cargo test --manifest-path crates/my-lisp-cli/Cargo.toml
cargo test --manifest-path crates/my-lisp-literate/Cargo.toml
```

## Deutsch

Jedes Rust-Crate unter `crates/` besitzt eigene automatisierte Tests, ausgeführt mit `cargo test --manifest-path crates/<crate>/Cargo.toml`. Ein crateübergreifendes Coverage-Werkzeug ist noch nicht eingerichtet; diese Tabelle ist die aktuelle Quelle der Wahrheit und sollte aktualisiert werden, sobald ein Crate Tests gewinnt oder verliert.

| Crate | Suite | Tests | Deckt ab |
|---|---|---:|---|
| `my-lisp` | Unit-Tests (`src/parser.rs`, `src/environment.rs`, `src/eval.rs`) | 24 | Reader-/Parser-Grenzfälle, Isolation des lexikalischen Scopes, Single-Pass-Auswertung, Makro-Expansion |
| `my-lisp` | `tests/mccarthy.rs` | 12 | die sieben McCarthy-Primitive, exakte/inexakte Arithmetik, Lambda-Semantik, strukturierte Fehler |
| `my-lisp` | `tests/stack_safety.rs` | 4 | Tail-Rekursion und Clone/Drop tiefer Listen mit konstantem Rust-Stack |
| `my-lisp-cli` | `tests/cli.rs` | 8 | die kompilierte Binärdatei durchgängig: `--version`/`--help`, Dateiausführung, Exit-Codes bei Parse-/Eval-Fehlern, fehlende Datei, Vorladen von `lib/core.my` |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | Offset-Zuordnung von literate-Markdown-Quellcode |
| **Gesamt** | | **52** | |

Letzter erfasster Lauf: 07.08.2026, Windows x86_64, Debug-Profil — alle 52 Tests bestanden, 0 fehlgeschlagen, 0 übersprungen.

Alles ausführen mit:

```powershell
cargo test --manifest-path crates/my-lisp/Cargo.toml
cargo test --manifest-path crates/my-lisp-cli/Cargo.toml
cargo test --manifest-path crates/my-lisp-literate/Cargo.toml
```
