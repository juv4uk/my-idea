# Пін SENS: точна драбина доменів (розробницький зріз)

**22.10.2026 · рішення власника:** IDE має споживати чинну доменну драбину, а не історичний SID8/Sens8 чи версію з попередньої моделі.

- `external/sens` → **`2052e3d6da9e2f79fe8ed57199b5b8eaa07600db`**, зафіксований SHA на `sens/main` (розробницький пін).
- Попередній gitlink: `12b7a75e4c602bfa3705dc17d176e3d9564d1fd8` (development-зріз від 19.10.2026).
- **Оновлення 22.10.2026:** пін переведено на поточний `sens/main` (`2052e3d6da9e`). Доменна драбина, Contract та єдиний gitlink лишаються чинними; `sens/main` — живий розробницький swarm, тож конкретний SHA фіксує зріз на цю дату.
- Новий пін зберігає виправлення `lib/reason.lisp` та усуває фактичні дефекти справжнього IDE smoke: неявна REPL-поверхня більше не ламає файловий CLI; знято хибний глобальний `block` у `meta-eval`; D9-аліаси `forward.lisp` визначаються після своїх функцій.
- Native GUI тепер викликає `sens_literate::eval_literate_ukr` із цього самого піна: парсинг, D24+ проєкція, обчислення й remap Markdown-діагностики належать SENS, без дубльованої GUI-реалізації.
- Точне D9 для української `рядок-додати` тепер походить зі згенерованої ратифікованої таблиці `lib/domains/d9.lisp`; у SENS lowering допускаються лише D9-голови з реальним string-механізмом. Це виправляє втрату лівого аргументу в IDE Editor API та не вводить SID8 міст.
- D9 `рядок-додати` тепер ідемпотентно зводиться до `DomainCall(D9)` уже на першому проході; це підтверджено справжнім runtime-тестом SENS. Плагіни Editor API, гачки подій та end-to-end приклади користуються ратифікованими `функція` і `рядок-додати` замість англійських голів.
- Джерело мовного закону: `external/sens/CURRENT.md`, `language-contract.lisp`, `lib/domains/d1.lisp` … `d10.lisp`. **Contract 11.9**.
- D1–D9 — ратифікований фундамент; D7 **v2 128/128**, D10 **цілий домен 1024/1024**; це НЕ 1024 доведені callable-функції. Числова драбина **D24+** — окрема.
- Людське `*.lisp`: канонічна українська проєкція `укр` із десятковими математичними числами. Машинний `*.sens`: фізичний **T5M** (Zig лише пакує точні слова). Файл без розширення — читабельний бітовий view, не машинний виконавний файл.
- **Семантика належить SENS**. GUI, Ratatui, Rust path crates, WASM, CLI і sidecar — лише споживачі одного exact gitlink. Жодних окремих SID8/Function8 джерел, floating clone або другого interpreter.
- `src-tauri/build.rs` і `scripts/build-repl-sidecar.sh` порівнюють **gitlink SHA** з checkout `external/sens` і зупиняють саме неспівпалу збірку замість мовчазного дрейфу.
- `scripts/check-my-lisp-sync.py` надалі перевіряє єдиний dependency channel, незалежно від історичної назви скрипта.

**Важлива межа:** це **development pin**, НЕ реліз `l0.42.0`, НЕ дозвіл на тегування й НЕ підтвердження повної міграції або зеленого `sens/main`. Upstream release #4196/#4198 зберігає власні докази й правила публікації. Після оновлення окремо потрібні реальні CI на цьому exact SHA, оновлений Cargo.lock і перевірка українського `Lisp → .sens` на чинному SENS; failure виправляти адресно, а не ставити новий HOLD.

Deutsch: Der Entwicklungs-Pin nutzt Contract 11.9 mit exakten D1–D10-Domänen und T5M; der Release-Status bleibt NO-GO.

English: The development pin follows the ratified domain ladder and one exact SENS gitlink; it is not a released l0.42.0 tag or proof of full runtime admission.
