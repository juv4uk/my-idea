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
  implementation), лишаючи найновіший запис на пару. Читає й опційне поле
  `environment` (`guix-revision`/`channels`/`manifest`) — додане в
  `evidence/README.md` за пропозицією my-idea (commit `c2080f6` у
  my-lisp), відсутнє на старіших записах без помилки.
- `git.rs::embedded_dependency_sha` — читає з `Cargo.lock` цього workspace
  pinned SHA embedded-двигуна (`my-lisp` як git-залежність), окремо від
  SHA сусіднього репо `my-lisp` на диску — щоб було видно розбіжність між
  "з чим зібраний застосунок" і "що зараз у робочій копії my-lisp".
- `mod.rs` — збирає все в `EcosystemStatus` (включно з `evidence_matrix`,
  `embedded_my_lisp_sha`), звіряє версії контрактів cml очікує проти
  версій, які my-lisp/fpga-lisp фактично надають
  (`CompatibilityCheck.language_match` / `isa_match`).

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

Frontend (`src-cljs/my_idea/`):

- `i18n.cljs` — messages/languages/themes/programming-language таблиці й
  `t`-lookup, винесені з `core.cljs` як перший крок його розбиття (файл
  переріс ~25KB і мав тенденцію ставати місцем для кожної нової фічі).
- Кнопка **🔭 Ecosystem** викликає `ecosystem_status` і рендерить панель
  замість AST/preview: branch/SHA + версія контракту на репо, SHA
  embedded-двигуна з попередженням про розбіжність, якщо він відрізняється
  від my-lisp на диску, language/ISA compatibility, і повна
  evidence-таблиця (G1–G8/S1–S3 × my-lisp/cml/fpga-lisp, ✓/✗/·, тултип із
  fixture/expected/actual/commit). Клік на рядок requirement відкриває
  fixture drill-down: джерело fixture плюс badge/expected/actual/commit/
  environment/note по кожній реалізації, з кнопкою "← matrix" назад.
- Кнопка **🔮 Oracle** шле вміст активного файлу як `eval`-запит до живого
  my-lisp TCP REPL і показує value при успіху або "oracle: error (kind) —
  message" при помилці — не сирий `(response ...)`.
- Кнопка **⚖ Compare** запускає той самий вираз паралельно через
  embedded-двигун (`evaluate_my_lisp`) і живий TCP oracle, показує обидва
  значення й позначає збіг/розбіжність — мінімальна версія
  "Run with ALL evaluators / SEMANTIC AGREEMENT" з vision-документа (поки
  лише два шляхи з трьох — CML→FPGA чекає на end-to-end fixture).
- Стилі `.eco-*` у `public/styles.css`, узгоджені зі світлою/темною/sepia
  темами через наявні CSS-змінні.

## Що ще НЕ реалізовано

- Timeline/Decisions-панелі з vision-документа, і граф проходження виразу
  через parser → cml → fpga (fixture drill-down уже є, повного графа
  немає).
- Кнопка "Run ecosystem check" (пункт 5 MVP-плану) досі читає лише
  контракти й evidence з диска — не запускає тести/CML/simulator сама.
- Compatibility Lens / вибір гілки для кожного репо — немає, показується
  лише поточний стан робочої копії.
- ⚖ Compare показує лише embedded-двигун vs oracle (два шляхи), не третій
  (CML→FPGA) — залежить від fpga-lisp/cml, не від `my-idea`.
- `core.cljs` (~25KB) все ще великий — виділено лише `i18n.cljs`;
  state/commands/views ще не розділені.

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
директорія лишається `/mnt/c/GitHub/my-idea`). rustc-версійний блокер
(Tauri-стек вимагав 1.86–1.88, поточний канал давав 1.85.1) вирішено
екосистемно: `my-lisp` зробив `guix pull` до каналу з rust 1.93.0 і додав
`channels.scm`/`evidence/README.md`'s `environment`-поле (commit `c2080f6`
у my-lisp). Пер-юзерний `guix pull` не поширюється автоматично на інших
користувачів WSL — `guix time-machine -C ../my-lisp/channels.scm -- shell
-m manifest.scm -- <command>` дає той самий канал без повторного `guix
pull`. `cargo check --workspace` для `my-idea` під цим каналом на момент
цього запису ще перевіряється (див. AGENTS.md за актуальною командою).

## Наступний крок

Найближча корисна ціль — не в `my-idea`: `cml` ще не провела `length`
наскрізно (`my-lisp` → `cml` → `fpga-lisp` → порівняння), а це перший
реальний end-to-end fixture, який evidence-матриця зможе показати повним
рядком по всіх трьох реалізаціях. Коли він з'явиться, наступний крок тут —
клікабельний drill-down з рядка матриці у fixture-панель (джерело,
expected/actual по кожній реалізації, note), перший крок до
"semantic debugger" з vision-документа.
