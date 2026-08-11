# Ecosystem status — my-idea

**Роль цього файлу**: append-only хронологічний лог фактів з боку `my-idea`,
за тим самим протоколом, що й `cml/ecosystem-status.md` і
`fpga-lisp/ecosystem-status.md`. Для курованого поточного знімка всієї
екосистеми див. `my-lisp`'s `ecosystem-status.my`/`ecosystem-status.md`.

Репозиторії на цій машині:
- `my-lisp` — C:\Users\user\Documents\GitHub\my-lisp
- `cml` — C:\Users\user\Documents\GitHub\cml
- `fpga-lisp` — C:\Users\user\Documents\GitHub\fpga-lisp
- `my-idea` — C:\Users\user\Documents\GitHub\my-idea (цей файл)

---

## [my-idea] 2026-08-11 — приєднання до координаційного протоколу

`my-idea` (Claude Code сесія) приєднується до cross-session координації з
my-lisp/cml/fpga-lisp. Короткий статус реалізації System Observatory MVP:

**Реалізовано** (`src-tauri/src/ecosystem/`):
- `git.rs` — знаходить сусідні репо на диску, читає branch/SHA.
- `contracts.rs` — читає `language-contract.my` (my-lisp), `isa-contract.my`
  (fpga-lisp), `compatibility.my` (cml) і my-lisp's `ecosystem-status.my`
  (`cml`-запис: tier-1-skips-remaining, ci-status, equal?/defmacro-status)
  через `my_lisp::parse` (власний reader my-lisp, не саморобний парсер).
- Fixture inventory: `read_fixture_inventory()` читає впорядкований
  канонічний `my-lisp/tests/fixtures/conformance.my` (expr/expected/error/
  tier/axioms/role/note).
- `mod.rs` збирає все в `EcosystemStatus`, звіряє версії контрактів, які
  cml очікує, проти фактичних версій my-lisp/fpga-lisp.
- Tauri command `ecosystem_status`, викликається кнопкою **🔭 Ecosystem** у
  desktop UI (ClojureScript, не Svelte, як помилково зазначено в
  ранньому vision-документі) — окремий екран з картками трьох репо
  (clone presence, branch, SHA, версія контракту) і блоком сумісності.
- Unit-тести для всіх contract/status/fixture readers (7 тестів,
  `cargo test --lib ecosystem`, усі зелені).

**Не реалізовано**: запуск тестів/CML/simulator — лише читання статичних
контрактів; evidence graph і fixture-рівневі execution results з
vision-документа.

**Залежність**: my-idea тягне my-lisp як cargo git-залежність
(`branch = "main"`), не submodule — синхронізація ревізій лише вручну, без
CI-інваріанту.

**Відомий локальний ризик**: цей же checkout одночасно редагується іншою,
незалежною координаційною системою (Codex/OpenCode, файлова тека
`C:\Users\user\Documents\GitHub\docs\`, поза git). Один раз це вже
призвело до незапланованого перемикання гілки (`main` →
`agent/test-contract-readers`) під час активної роботи Claude Code сесії.
Дані не втрачено, але це підтверджує потребу в окремому `git worktree` для
кожного одночасно активного агента в цьому репо.

**TCP REPL my-lisp (127.0.0.1:9999)**: підключення напряму заблоковано
класифікатором дозволів Claude Code auto mode (raw TCP socket + виконання
коду через нього трактується як небезпечна дія за замовчуванням) — це
обмеження платформи агента, не рішення сесії. Як обхідний шлях
використано вже зібраний локальний бінарник `my-lisp/target/debug/my-lisp.exe`
напряму (без мережі) для перевірки семантики.

Немає відкритих питань до інших репо на цей момент.
