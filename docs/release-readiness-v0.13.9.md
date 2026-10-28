# Підготовка релізу my-idea v0.13.9

## Українська

### 28.10.2026 · виправлення 100k WASM — інтеграційний кандидат

- SENS gitlink `external/sens` тепер `812851edc86a3bc483d326756593c66eed345ba8` (D3 COND tail trampoline). **Це не релізний SENS тег**, тому цей commit `my-idea` призначений для CI перевірки без права публікації.
- Раніше після SENS `s0.42.2` були **42/45 Node/Playwright tests**: три невдачі стосувалися stack overflow на 100k-list у WASM і portable Web. Перевірки залишено незмінними.
- Тег `v0.13.9` залишається заблокованим до зеленого exact-head CI `my-idea`, повного release evidence та наступного опублікованого upstream SENS SHA.

### Оновлення 28.10.2026 — SENS pin для v0.13.9

- Upstream SENS `s0.42.2` опубліковано на `f800db52d3f3c92d9573b8822fdf771b807aabc4`; [CI](https://github.com/juv4uk/sens/actions/runs/40644786978), [release-preflight](https://github.com/juv4uk/sens/actions/runs/40644419991) і [release](https://github.com/juv4uk/sens/actions/runs/40645450709) завершилися успішно на цьому SHA.
- `external/sens` оновлено до цього exact SHA; `Cargo.lock` синхронізовано для `sens`, `sens-host`, `sens-literate` версії `0.42.2` та `sens-host → serde_json`.
- **Реліз IDE блокується до зеленого hosted CI на точному новому SHA my-idea**; тег `v0.13.9` публікувати тільки після перевірок і комплекту артефактів.
- Нижче збережено історичний стан на 25.10.2026, який стосувався старого gitlink `684e1878`.

### Історичний стан 25.10.2026

**Статус: підготовлено до фінального етапу, але НЕ ГОТОВО до публікації.** Тег `v0.13.9` не створювати до проходження SENS release gate і зеленого hosted CI цього репозиторію.

### Перевірена база на 25.10.2026

- Репозиторій: juv4uk/my-idea.
- Актуальний `main`: `32faf8361b2316c0c96039d12d4d3f499ea85465`.
- `package.json` і `src-tauri/Cargo.toml` мають версію `0.13.9`.
- Gitlink `external/sens`: `684e1878281835a269aca431654dccae40cd5d91`.
- Тримовні нотатки готові: `.github/release-notes/v0.13.9.md` (English / Українська / Deutsch).
- Embedded Rust, WASM і desktop CLI sidecar мають походити з одного точного gitlink SHA; publish workflow перевіряє його через `scripts/check-my-lisp-sync.py`.
- Останній опублікований реліз — `v0.13.8`. Перевірка Git ref для `v0.13.9` повертає NOT_FOUND: тег ще не створений.

### Поточний блокер hosted CI

Актуальний hosted CI [#40238936867](https://github.com/juv4uk/my-idea/actions/runs/40238936867) на exact SHA `32faf8361b23` завершився failure на `bun run test`.

- Node tests: **45 загалом, 25 пройшли, 20 впали**.
- 18 conformance-прикладів і два WASM/standalone deep-list сценарії не можуть завантажити `time.lisp`.
- Runtime-помилка: `exact DEFINE target may install a language-owned mechanism only at the root frame`, `physical_form_index=0`, `physical_word_start=0`, `physical_word_end=1359`.
- Root cause — застарілий SENS gitlink `684e187`, у якому `lib/time.lisp` використовує старе визначення числових коренів через D6 LENGTH. В історії SENS є міграція констант на D24+ (`2663841d757c` та подальші зміни), однак це ще не доказ релізної готовності поточного SENS main.

Не видаляти conformance cases, не вимикати deep-list tests, не послаблювати DEFINE policy й не підміняти SENS семантику в IDE. Спершу SENS має отримати зелений exact-SHA CI та канонічні фізичні пари.

### Що вже підготовлено

- Версійні поля синхронізовані на `0.13.9`.
- Release notes описують SENS editing loop, portable `my-idea-web.html`, Editor API/plugins, TUI й cross-platform sidecar pin.
- Publish workflow має окрему валідацію тегу, точного gitlink та єдиного джерела для embedded Rust/WASM/sidecar.
- GUI Run/Stop і CML Compile чесно залишені поза стабільними обіцянками; TUI не замінює GUI acceptance.
- Старий успішний CI `#39988141019` на SHA `99ba5e6cf704` підтверджує лише той попередній зріз, не поточний `32faf836`.

### Обов'язковий порядок релізу

1. У `juv4uk/sens` усунути RED, отримати повний нативний зелений CI на одному SHA, підтвердити F6 producer → physical T5M → consumer і фізичний паритет.
2. Власник затверджує точний SENS release SHA/тег та дозвіл на публікацію. Поточний SENS main не має підтвердженого релізного GO.
3. У `my-idea` змінити лише gitlink `external/sens` на підтверджений SENS SHA одним окремим комітом. Не пінити непідтверджений main і не додавати другий Cargo git-source.
4. Запустити hosted CI для точного my-idea SHA: pin guard, Bun build, portable Web IDE artifact, `bun run test`, `bun run check`, Rust witness tests і платформні release preflights.
5. Лише після зеленого CI створити `v0.13.9`; перевірити фактичні assets Linux, macOS universal, Windows, ARM Linux, Android і portable `my-idea-web.html`.

### Чекпоінт

- my-idea current SHA: `32faf8361b2316c0c96039d12d4d3f499ea85465`.
- SENS gitlink: `684e1878281835a269aca431654dccae40cd5d91`.
- Current my-idea CI: [#40238936867 — failure на bun run test](https://github.com/juv4uk/my-idea/actions/runs/40238936867).
- SENS main під час підготовки: `63466a8a095a83dccccdaff3a2e27b46e03c2aa6`; перевірені CI містили незавершені та/або failed runs. Це не затверджений pin.
- `v0.13.9` tag: не створений.
- Рішення: **NO-GO до завершення SENS → pin verified SHA → повний my-idea CI**.

## Deutsch

**Vorbereitet, nicht zur Veröffentlichung freigegeben.** Der aktuelle my-idea-Commit `32faf8361b23` schlägt beim `bun run test` fehl: 20 von 45 Tests können wegen der veralteten SENS-Untermodulrevision `684e187` `time.lisp` nicht laden. Version `0.13.9`, dreisprachige Release Notes und der Gitlink-Prüfpfad sind vorhanden; ein `v0.13.9`-Tag existiert noch nicht. Erst SENS mit grünem Exact-SHA-CI und synchronisierten T5M-Artefakten freigeben, danach genau diesen verifizierten SHA pinnen und my-idea vollständig testen.

## English

**Prepared, not release-ready.** The current my-idea commit `32faf8361b23` fails `bun run test`: 20 of 45 tests cannot load `time.lisp` because the recorded SENS submodule pin `684e187` is stale. Version metadata, tri-lingual release notes, and the single-pin validation workflow are present; no `v0.13.9` tag exists. First obtain an approved SENS release SHA with green exact-SHA CI and synchronized T5M artifacts, then pin that verified SHA and require a complete green my-idea hosted CI before tagging.
