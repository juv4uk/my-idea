# System Observatory — поточний стан реалізації в my-idea

Технічний статус-звіт, доповнення до [system-observatory-vision.md](system-observatory-vision.md)
(бачення користувача, не редагувати). Цей файл фіксує, що з MVP-плану вже
реалізовано, і призначений звірятись з `ecosystem-status.my`'s
`repositories.my-idea` записом у `my-lisp`.

## Що реалізовано (2026-08-11)

Backend-агрегатор у `src-tauri/src/ecosystem/`:

- `git.rs` — знаходить сусідні репо (`my-lisp`, `fpga-lisp`, `cml`) поруч із
  `my-idea` на диску (`~/Documents/GitHub/{my-idea,my-lisp,fpga-lisp,cml}`,
  offline-first), читає їхню поточну гілку й SHA через `git`.
- `contracts.rs` — парсить машинно-читані контракти власним reader'ом
  `my-lisp` (crate `my_lisp`, а не саморобний рядковий парсер):
  - `language-contract.my` (my-lisp)
  - `isa-contract.my` (fpga-lisp)
  - `compatibility.my` (cml: версія компілятора + версії language/ISA
    контрактів, з якими він тестувався)
  - `ecosystem-status.my` (my-lisp: `cml`-запис — `tier-1-skips-remaining`,
    `ci-status`, `equal-status`, `defmacro-status`)
- `mod.rs` — збирає все в `EcosystemStatus`, звіряє версії контрактів cml
  очікує проти версій, які my-lisp/fpga-lisp фактично надають
  (`CompatibilityCheck.language_match` / `isa_match`).
- Fixture inventory — читає впорядковані записи з канонічного sibling-файлу
  `my-lisp/tests/fixtures/conformance.my` через reader `my_lisp` і повертає
  `expr`, `expected`/`error`, `tier`, `axioms`, `role`, `note`. Відсутній або
  невалідний файл дає порожній список, не ламаючи решту status-відповіді.

Викликається наскрізь заново при кожному запиті — без кешування, тому
результат завжди відповідає стану на диску.

## Видимий MVP

- `ecosystem_status` зареєстровано як Tauri-команду та викликається кнопкою **🔭 Ecosystem**.
- Окремий екран показує три картки: `my-lisp`, `cml`, `fpga-lisp` — наявність локального clone, branch, SHA і версію відповідного контракту/компілятора.
- Блок сумісності окремо показує збіг language contract та ISA contract; повторний **Run ecosystem check** читає стан із диска заново.
- Екран адаптується до вузького вікна й використовує наявні теми IDE.

## Що ще НЕ реалізовано

- Не запускає тести/CML/simulator — лише читає статичні контракти.
  Кнопка оновлює агрегований знімок, але ще не запускає pipeline.
- Не читає `fpga-lisp`'s hardware-verified-milestones чи fixture-рівневі
  результати з `ecosystem-status.my`: inventory контракту вже є, але статусів
  виконання Rust/FPGA/CML та evidence ще немає.

## Залежність від my-lisp

`my-idea` тягне `my-lisp` як **cargo git-залежність** (`Cargo.toml`,
`branch = "main"`), не як git submodule. Ризик синхронізації ревізій:
**лише вручну** — немає CI-інваріанту, що перевіряє, чи зафіксована версія
`my-lisp` у `Cargo.lock` сумісна з тим, що `compatibility.my`/
`ecosystem-status.my` вважають актуальним. Розбіжність можлива, якщо
`my-lisp` зробить breaking change в reader/`Expr`, а `my-idea` не оновить
lockfile.

## Наступний крок

Додати fixture-рівневі результати та evidence: запуск Rust/CML/simulator,
три колонки результатів і перехід від кожного статусу до його доказу.
