# Підготовка релізу my-idea v0.13.9

## Українська

**Стан: підготовлено, але NO-GO до завершення релізу SENS і повторного зеленого hosted CI. Жодного тегу чи публікації не створено.**

### Перевірена база

- Версія в `package.json` та `src-tauri/Cargo.toml`: `0.13.9`.
- Поточний main під час аудиту: `1cc187a52895a691ff019272c5df5f8598444053`.
- Підмодуль `external/sens`: gitlink `684e1878281835a269aca431654dccae40cd5d91`.
- Release notes: `.github/release-notes/v0.13.9.md`, три мови (English / Українська / Deutsch).
- Останній опублікований реліз: `v0.13.8`. Тег `v0.13.9` не існує.
- Workflow публікації перевіряє точний gitlink через `scripts/check-my-lisp-sync.py`, а desktop-пакети, WASM і CLI sidecar мають походити з одного pin.

### Блокер hosted CI на поточному SHA

[Run #40237456501](https://github.com/juv4uk/my-idea/actions/runs/40237456501) завершився failure на `bun run test`. Результат Node tests: 45 тестів, 25 пройшли, 20 не пройшли. Найбільша спільна причина — 18 conformance-прикладів і два WASM/standalone deep-list сценарії не можуть завантажити `time.lisp`:

```
exact DEFINE target may install a language-owned mechanism only at the root frame
physical_form_index=0 physical_word_start=0 physical_word_end=1359
```

Це не виправляється оновленням нотаток або зниженням очікувань тестів. У pinned SENS tree `684e187` використовується старіша `lib/time.lisp`; історія SENS має пізніше виправлення `2663841d757c` (D24 constants замість D6 LENGTH у time/vector/knowledge). Його наявність саме по собі ще не є доказом, що помилку усунуто — потрібні зелений CI SENS і точний новий submodule pin.

### Release notes і функції

Нотатки v0.13.9 описують робочий SENS editing loop, portable `my-idea-web.html`, єдиний pin SENS, Editor API/plugins, опціональний Ratatui TUI й платформне пакування. Вони також чітко зазначають, що GUI Run/Stop і CML Compile не рекламуються як стабільні. Не видаляти ці межі та не називати TUI заміною GUI acceptance.

### Точний порядок релізу

1. У `juv4uk/sens` довести релізний кандидат: повний нативний CI green на одному SHA, фізичні пари синхронізовані, F6 producer → physical T5M → consumer проходить; власник підтверджує SHA і дозволяє тег.
2. У цьому репозиторії оновити тільки gitlink `external/sens` до підтвердженого коміту/тегу SENS. Не рухати gitlink на непідтверджений main і не додавати паралельний Cargo git-source.
3. Запустити весь hosted CI для нового exact my-idea SHA. Обов’язкові кроки: `check-my-lisp-sync.py`, `bun run build`, портативний `my-idea-web.html`, `bun run test`, `bun run check`, Rust witness tests.
4. Лише після green CI та наявності web artifact створювати тег `v0.13.9`. Publish workflow збирає Linux, macOS universal, Windows, ARM Linux та Android release assets; після публікації перевірити реальні артефакти й release page.

### Checkpoint для наступного агента

- Не створювати `v0.13.9` поки.
- Спершу дочекатися підтвердженого SENS release SHA, потім оновити gitlink у my-idea одним комітом.
- Якщо помилка `time.lisp` лишається після pin оновлення, виправляти SENS source/runtime, а не викидати conformance приклади чи вимикати depth tests.
- Після кожного точкового коміту перевіряти exact-SHA CI; історичний успіх на іншому SHA не переноситься.

## Deutsch

**Vorbereitet, aber noch nicht freigegeben.** Version ist `0.13.9`, Release Notes sind dreisprachig und der Publish-Workflow prüft den exakten SENS-Gitlink. Der aktuelle Hosted-CI-Lauf schlägt beim Laden von `time.lisp` fehl; deshalb kein `v0.13.9`-Tag. Erst SENS mit grünem Exact-SHA-CI und synchronisierten T5M-Artefakten veröffentlichen, danach den bestätigten Gitlink pinnen und den vollständigen my-idea-Hosted-CI erfolgreich ausführen.

## English

**Prepared, not release-ready.** Version metadata is `0.13.9`; tri-lingual notes and the publish workflow are present. Current hosted CI fails while loading `time.lisp` (20 of 45 Node tests failed because the pinned SENS runtime rejects an exact DEFINE target). No `v0.13.9` tag was created. First release SENS only after exact-SHA native CI and physical T5M parity pass; then pin that verified revision in my-idea and require a clean full hosted CI plus the portable web artifact before tagging.
