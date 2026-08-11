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

Викликається наскрізь заново при кожному запиті — без кешування, тому
результат завжди відповідає стану на диску.

## Що ще НЕ реалізовано

- Немає `#[tauri::command]` обгортки — `status()` поки не викликається з
  фронтенду. Немає Svelte UI (System Map, fixture-панелі, evidence graph
  тощо з vision-документа).
- Не запускає тести/CML/simulator — лише читає статичні контракти.
  Пункт 5 MVP-плану ("кнопка Run ecosystem check") не почато.
- Не читає `fpga-lisp`'s hardware-verified-milestones чи fixture-рівневі
  дані з `ecosystem-status.my` — лише `cml`-запис.

## Залежність від my-lisp

`my-idea` тягне `my-lisp` як **cargo git-залежність** (`Cargo.toml`,
`branch = "main"`), не як git submodule. Ризик синхронізації ревізій:
**лише вручну** — немає CI-інваріанту, що перевіряє, чи зафіксована версія
`my-lisp` у `Cargo.lock` сумісна з тим, що `compatibility.my`/
`ecosystem-status.my` вважають актуальним. Розбіжність можлива, якщо
`my-lisp` зробить breaking change в reader/`Expr`, а `my-idea` не оновить
lockfile.

## Наступний крок

Додати `#[tauri::command] fn ecosystem_status() -> EcosystemStatus` і
мінімальний Svelte-екран, що показує три колонки (branch/SHA, contract
versions, compatibility match) — перший видимий шматок MVP-плану.
