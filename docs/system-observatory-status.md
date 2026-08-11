# System Observatory — поточний стан реалізації в my-idea

Технічний статус-звіт, доповнення до [system-observatory-vision.md](system-observatory-vision.md)
(бачення користувача, не редагувати). Цей файл фіксує, що з MVP-плану вже
реалізовано, і призначений звірятись з `ecosystem-status.my`'s
`repositories.my-idea` записом у `my-lisp`.

## Що реалізовано (2026-08-12)

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
- `evidence.rs` — сканує `evidence/<requirement>/<implementation>/*.my` у
  всіх трьох сусідніх репо (my-lisp, fpga-lisp, cml), парсить кожен запис
  тим самим reader'ом, і зводить їх у requirement-матрицю (G1–G8/S1–S3 ×
  implementation), лишаючи найновіший запис на пару.
- `mod.rs` — збирає все в `EcosystemStatus` (включно з `evidence_matrix`),
  звіряє версії контрактів cml очікує проти версій, які my-lisp/fpga-lisp
  фактично надають (`CompatibilityCheck.language_match` / `isa_match`).

Викликається наскрізь заново при кожному запиті — без кешування, тому
результат завжди відповідає стану на диску.

`src-tauri/src/oracle.rs` — одноразовий TCP-клієнт до my-lisp's
`--tcp --protocol=sexpr` REPL: одне з'єднання на запит (той самий oracle,
що й evidence-протокол, не message bus — див. my-lisp AGENTS.md), з
ретраями на з'єднання (оракул міг щойно стартувати). Відповідь парситься
тим самим reader'ом my-lisp (status/kind/message), а не показується як
сирий `(response ...)`.

Обидва модулі підключені як `#[tauri::command]`:

- `ecosystem_status` — читає статичний стан із диска.
- `oracle_query(source, op, port)` — запитує живий my-lisp TCP REPL
  (`eval` за замовчуванням, порт 9999 за замовчуванням).

Frontend (`src-cljs/my_idea/core.cljs`):

- Кнопка **🔭 Ecosystem** викликає `ecosystem_status` і рендерить панель
  замість AST/preview: branch/SHA + версія контракту на репо,
  language/ISA compatibility, і повна evidence-таблиця (G1–G8/S1–S3 ×
  my-lisp/cml/fpga-lisp, ✓/✗/·, тултип із fixture/expected/actual/commit).
  Клік на рядок requirement відкриває fixture drill-down: джерело fixture
  плюс badge/expected/actual/commit/note по кожній реалізації, з кнопкою
  "← matrix" назад.
- Кнопка **🔮 Oracle** шле вміст активного файлу як `eval`-запит до живого
  my-lisp TCP REPL і показує value при успіху або "oracle: error (kind) —
  message" при помилці — не сирий `(response ...)`.
- Стилі `.eco-*` у `public/styles.css`, узгоджені зі світлою/темною/sepia
  темами через наявні CSS-змінні.

## Що ще НЕ реалізовано

- Fixture-рівневий Graph/Timeline/Decisions-панелі з vision-документа —
  зроблено лише плоска requirement-матриця, без drill-down у конкретний
  fixture, без графа проходження виразу через parser → cml → fpga.
- Кнопка "Run ecosystem check" (пункт 5 MVP-плану) досі читає лише
  контракти й evidence з диска — не запускає тести/CML/simulator сама.
- Compatibility Lens / вибір гілки для кожного репо — немає, показується
  лише поточний стан робочої копії.

## Залежність від my-lisp

`my-idea` тягне `my-lisp` як **cargo git-залежність** (`Cargo.toml`,
`branch = "main"`), не як git submodule. Ризик синхронізації ревізій:
**лише вручну** — немає CI-інваріанту, що перевіряє, чи зафіксована версія
`my-lisp` у `Cargo.lock` сумісна з тим, що `compatibility.my`/
`ecosystem-status.my` вважають актуальним. Розбіжність можлива, якщо
`my-lisp` зробить breaking change в reader/`Expr`, а `my-idea` не оновить
lockfile.

## Середовище розробки

`my-idea` тепер працює з WSL2 + Guix (`manifest.scm` у корені репо,
`guix shell -m manifest.scm`, користувач `my-idea` в WSL Ubuntu, робоча
директорія лишається `/mnt/c/GitHub/my-idea`). Відомий блокер:
Guix-канал дає `rust` 1.85.1, а кілька транзитивних залежностей
Tauri-стека (`darling`, `icu_*`, `time`, `zbus`, `plist`) вимагають
1.86–1.88 — `cargo check --workspace` не проходить, доки це не вирішено
(`guix pull` на новіший канал або точкове пінування версій крейтів).

## Наступний крок

Найближча корисна ціль — не в `my-idea`: `cml` ще не провела `length`
наскрізно (`my-lisp` → `cml` → `fpga-lisp` → порівняння), а це перший
реальний end-to-end fixture, який evidence-матриця зможе показати повним
рядком по всіх трьох реалізаціях. Коли він з'явиться, наступний крок тут —
клікабельний drill-down з рядка матриці у fixture-панель (джерело,
expected/actual по кожній реалізації, note), перший крок до
"semantic debugger" з vision-документа.
