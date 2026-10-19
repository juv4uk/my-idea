# Пін SENS: точна драбина доменів (розробницький зріз)

**19.10.2026 · рішення власника:** IDE має споживати чинну доменну драбину, а не історичний SID8/Sens8 чи версію з попередньої моделі.

- `external/sens` → **`5e9d1d2132a29520280170b2c7212b4239b3956d`**, зафіксований SHA на `sens/main` (розробницький пін).
- Попередній gitlink: `ca0d984c6cecf589d0da6832191f1b5fdfca5709` (старіший development-зріз).
- Новий пін включає цільове виправлення `lib/reason.lisp`: локальний індекс правил більше не змішується з D9 і не має невизначеного alias `index`.
- Джерело мовного закону: `external/sens/CURRENT.md`, `language-contract.lisp`, `lib/domains/d1.lisp` … `d10.lisp`. **Contract 11.9**.
- D1–D9 — ратифікований фундамент; D7 **v2 128/128**, D10 **цілий домен 1024/1024**; це НЕ 1024 доведені callable-функції. Числова драбина **D24+** — окрема.
- Людське `*.lisp`: канонічна українська проєкція `укр` із десятковими математичними числами. Машинний `*.sens`: фізичний **T5M** (Zig лише пакує точні слова). Файл без розширення — читабельний бітовий view, не машинний виконавний файл.
- **Семантика належить SENS**. GUI, Ratatui, Rust path crates, WASM, CLI і sidecar — лише споживачі одного exact gitlink. Жодних окремих SID8/Function8 джерел, floating clone або другого interpreter.
- `src-tauri/build.rs` і `scripts/build-repl-sidecar.sh` порівнюють **gitlink SHA** з checkout `external/sens` і зупиняють саме неспівпалу збірку замість мовчазного дрейфу.
- `scripts/check-my-lisp-sync.py` надалі перевіряє єдиний dependency channel, незалежно від історичної назви скрипта.

**Важлива межа:** це **development pin**, НЕ реліз `l0.42.0`, НЕ дозвіл на тегування й НЕ підтвердження повної міграції або зеленого `sens/main`. Upstream release #4196/#4198 зберігає власні докази й правила публікації. Після оновлення окремо потрібні реальні CI на цьому exact SHA, оновлений Cargo.lock і перевірка українського `Lisp → .sens` на чинному SENS; failure виправляти адресно, а не ставити новий HOLD.

Deutsch: Der Entwicklungs-Pin nutzt Contract 11.9 mit exakten D1–D10-Domänen und T5M; der Release-Status bleibt NO-GO.

English: The development pin follows the ratified domain ladder and one exact SENS gitlink; it is not a released l0.42.0 tag or proof of full runtime admission.
