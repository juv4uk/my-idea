# Локальні GitHub Actions ранери · my-idea

## Чинна маршрутизація (рішення власника · 19.10.2026)

**Ratatui/TUI перевіряється на локальних Linux/X64 repo-scoped runner-ах.**
`.github/workflows/tui.yml` використовує `runs-on: [self-hosted, Linux, X64]`.
Додаткова мітка `core-local` не потрібна для цього job: вона обмежувала планувальник, не будучи підтвердженою.

- TUI job запускає справжні тести/збірку Rust та file-mode/REPL SENS зі строго pinned `external/sens`.
- Для економії CPU використовується одна concurrency group на гілку і `CARGO_BUILD_JOBS=2`.
- `.github/workflows/ci.yml` залишається на GitHub-hosted Ubuntu для P0 Tauri/Playwright, якщо не надійде окреме рішення про його перенесення.
- `.github/workflows/publish-release.yml` залишається на GitHub-hosted Linux/Windows/macOS; переносні релізи не запускати без потреби на локальному ПК.
- Жодна зміна runner routing сама по собі не змінює gitlink `external/sens` або семантику SENS.

## Фактичний стан та докази

Доступний GitHub Actions job не означає, що runner зареєстрований або почав працювати.
Для конкретного SHA у GitHub Actions потрібно перевірити:

1. `labels`: `self-hosted`, `Linux`, `X64`; `runner_name` — назва реально призначеного локального runner-а.
2. `status`: queued ≠ in_progress ≠ completed/success. Не називати CI зеленим до завершення.
3. Логи Cargo і SENS містять справжній SHA `PINNED_SENS_GITLINK` та `PINNED_SENS_*_OK`, а не нуль тестів.

Якщо job лишається queued — перевірити **Settings → Actions → Runners** саме цього репозиторію та Linux/X64 labels на зареєстрованих runner-ах.
Доступність Desktop Commander не є свідком доступності GitHub Actions runner.
Не запускати повторно старі червоні workflows, не вимикати TLS, не публікувати реєстраційні токени.

---

**Deutsch:** Ratatui CI verwendet einen lokalen self-hosted Linux/X64-Runner; Release-Matrizen bleiben GitHub-hosted.

**English:** Ratatui CI uses local self-hosted Linux/X64; release jobs remain GitHub-hosted.
