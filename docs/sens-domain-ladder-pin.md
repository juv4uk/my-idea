# Пін SENS: реліз s0.42.2 і точна драбина доменів

**28.10.2026 · поточний релізний пін:** зафіксовано SENS [`s0.42.2`](https://github.com/juv4uk/sens/releases/tag/s0.42.2) на `f800db52d3f3c92d9573b8822fdf771b807aabc4` (Contract 12.0). Upstream exact-head [CI](https://github.com/juv4uk/sens/actions/runs/40644786978), [preflight](https://github.com/juv4uk/sens/actions/runs/40644419991) та [release](https://github.com/juv4uk/sens/actions/runs/40645450709) — SUCCESS. Інтеграція `my-idea` потребує власного зеленого hosted CI перед тегом `v0.13.9`.

**22.10.2026 · історія:** IDE має споживати чинну доменну драбину, а не історичний SID8/Sens8 чи версію з попередньої моделі.

- `external/sens` → **`f800db52d3f3c92d9573b8822fdf771b807aabc4`**, точний релізний gitlink SENS `s0.42.2`.
- Попередній gitlink: `684e1878281835a269aca431654dccae40cd5d91` (development-зріз від 23.10.2026).
- **Оновлення 23.10.2026:** пін переведено на `sens/main` (`684e1878`) — там **подання дерева розбору винесено в бібліотеку `sens`** (`presentation.rs`: `render_ast`/`render_ast_node`/`AstNode`/`AstNodeKind`/`ast_nodes`). CLI (`:розбір`/`:розбір-виконати`) і TUI тепер мають **одну** реалізацію обходу й класифікації вузлів; підписи лишаються за хостом (TUI — uk→de→en), тож жодного дублю обходу. Вивід CLI ідентичний.
- **Оновлення 22.10.2026:** пін переведено на поточний `sens/main` (`2052e3d6da9e`). Доменна драбина, Contract та єдиний gitlink лишаються чинними; `sens/main` — живий розробницький swarm, тож конкретний SHA фіксує зріз на цю дату.
- Новий пін зберігає виправлення `lib/reason.lisp` та усуває фактичні дефекти справжнього IDE smoke: неявна REPL-поверхня більше не ламає файловий CLI; знято хибний глобальний `block` у `meta-eval`; D9-аліаси `forward.lisp` визначаються після своїх функцій.
- Native GUI тепер викликає `sens_literate::eval_literate_ukr` із цього самого піна: парсинг, D24+ проєкція, обчислення й remap Markdown-діагностики належать SENS, без дубльованої GUI-реалізації.
- Точне D9 для української `рядок-додати` тепер походить зі згенерованої ратифікованої таблиці `lib/domains/d9.lisp`; у SENS lowering допускаються лише D9-голови з реальним string-механізмом. Це виправляє втрату лівого аргументу в IDE Editor API та не вводить SID8 міст.
- D9 `рядок-додати` тепер ідемпотентно зводиться до `DomainCall(D9)` уже на першому проході; це підтверджено справжнім runtime-тестом SENS. Плагіни Editor API, гачки подій та end-to-end приклади користуються ратифікованими `функція` і `рядок-додати` замість англійських голів.
- Джерело мовного закону: `external/sens/CURRENT.md`, `language-contract.lisp`, `lib/domains/d1.lisp` … `d10.lisp`. **Contract 12.0**.
- D1–D9 — ратифікований фундамент; D7 **v2 128/128**, D10 **цілий домен 1024/1024**; це НЕ 1024 доведені callable-функції. Числова драбина **D24+** — окрема.
- Людське `*.lisp`: канонічна українська проєкція `укр` із десятковими математичними числами. Машинний `*.sens`: фізичний **T5M** (Zig лише пакує точні слова). Файл без розширення — читабельний бітовий view, не машинний виконавний файл.
- **Семантика належить SENS**. GUI, Ratatui, Rust path crates, WASM, CLI і sidecar — лише споживачі одного exact gitlink. Жодних окремих SID8/Function8 джерел, floating clone або другого interpreter.
- `src-tauri/build.rs` і `scripts/build-repl-sidecar.sh` порівнюють **gitlink SHA** з checkout `external/sens` і зупиняють саме неспівпалу збірку замість мовчазного дрейфу.
- `scripts/check-my-lisp-sync.py` надалі перевіряє єдиний dependency channel, незалежно від історичної назви скрипта.

**Важлива межа:** тепер це опублікований SENS release pin, але не автоматичний дозвіл на випуск IDE. Після оновлення gitlink і Cargo.lock необхідні окремі hosted CI, Web IDE smoke, desktop sidecar та релізні перевірки саме для нового commit my-idea. Старі development-факти вище — історія.

Deutsch: Der Release-Pin verwendet SENS s0.42.2 und Contract 12.0; die IDE-Freigabe benötigt eigene erfolgreiche CI.

English: The gitlink now pins SENS s0.42.2 exactly. This is not a claim of full runtime admission or IDE release approval.
