# GitHub-hosted CI · Ранери GitHub · GitHub-Runner

## Чинний порядок (19.10.2026)

**`juv4uk/my-idea` використовує GitHub-hosted ранери, а не локальний WSL/self-hosted runner.** Достатньо стандартних GitHub Actions; реєструвати ранери на комп'ютері власника чи запускати `./run.sh` не потрібно.

- `.github/workflows/ci.yml`: перевірки `main` (`push`), pull request і ручний `workflow_dispatch`; job `hosted-web` на **`ubuntu-24.04`**.
- `.github/workflows/tui.yml`: збірка Ratatui на **`ubuntu-24.04`** за зміни `my-idea-tui/**` / самого workflow на `main`, або вручну.
- `.github/workflows/publish-release.yml`: релізні jobs на GitHub-hosted Ubuntu, Windows і macOS. Конкретну платформу визначає `runs-on` у job або `matrix.platform`.
- Інші наявні workflow теж мають використовувати лише GitHub-hosted `runs-on`; не повертайте `[self-hosted, linux, x64]`, WSL/Guix як умову розміщення job чи прив'язку до постійно ввімкненого ПК.

Пріоритет — **один змістовний автоматичний CI для кожного коміту**, із реальними SENS-перевірками, без дубльованих запусків або фіктивних зелених статусів. У `ci.yml` push/PR перевіряються на одному hosted job, а точний CML witness запускається через `workflow_dispatch`.

## Як перевірити стан

Відкрийте [GitHub Actions](https://github.com/juv4uk/my-idea/actions) і перевірте runs **CI · Перевірка · Prüfung** та **TUI · my-idea-tui · Ratatui** для потрібного SHA. Переконайтеся, що job отримав GitHub-hosted runner, пройшов усі кроки та завершився `success`. Якщо run `queued`, `failure` або відсутній — не називайте збірку зеленою; з'ясуйте причину у логах GitHub Actions.

Збірка в TUI workflow — це **перевірка компіляції**, а не доказ наскрізного виконання SENS-файла. Поведінка `Open → Edit → Save → Run → Output → Stop` потребує окремого реального smoke на тому самому pinned `external/sens`.

## Доступ до підмодулів

Основний CI читає зафіксовані gitlink-и. Для приватних залежностей використовуйте репозиторні secrets `SUBMODULE_READ_TOKEN` або `SENS_READ_TOKEN` із мінімальними правами на читання; якщо доступ дозволяє стандартний `GITHUB_TOKEN`, окремий секрет не потрібен. **Не записуйте токени у файли, workflow або команди Git.**

## Раніше встановлений локальний runner

Сам по собі перехід workflow на `ubuntu-24.04` **не видаляє** реєстрацію self-hosted runner у налаштуваннях GitHub чи службу на ПК. Якщо ранер був зареєстрований, власник може перевірити його в **Repository Settings → Actions → Runners** і видалити після підтвердження, що жодний чинний workflow його не потребує. Це окрема операція, не передумова hosted CI.

---

**Deutsch:** Alle regulären CI- und TUI-Jobs laufen auf GitHub-hosted Runnern. Es ist keine Registrierung eines lokalen WSL-Runners erforderlich. Ein bereits registrierter Runner wird dadurch nicht automatisch entfernt.

**English:** Regular CI and TUI jobs run on GitHub-hosted runners. No local WSL runner registration is required. Existing self-hosted runner registrations are not removed automatically.
