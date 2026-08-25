# Architecture Recovery Review — my-idea

**Дата:** 2026-08-25 · **Автор:** Vyasa (COMPILER STEWARD)
**Тип:** read-only recovery review · **Задача:** ARCH-RECOVERY-REVIEW-IDE
**Ресурси:** читання; жодних білдів (bun/tauri не запускав)

---

## 1. As-built

```
Tauri 2 desktop+mobile      src-tauri/ (757 LOC Rust):
  lib.rs / main.rs            вхідні точки
  oracle.rs                   клієнт семантичного оракула (:9999)
  swarm.rs                    клієнт mesh-реєстру
  swarm_dashboard.rs          дашборд стану рою
Frontend: shadow-cljs (src-cljs/my_idea) + bun.lock toolchain
Інтеграції: ecosystem/, external/, prototype/, memory/
Тести: tests/*.test.mjs ×3; scripts: dev/build/wasm/check/test/benchmark/tauri
Версія: v0.13.1 (Android safe-area release); 24 комміти за 7 днів
```

## 2. Роль у екосистемі [підтверджено кодом]
Профільна формула «observatory/read-only UI» підтверджується: oracle.rs +
swarm*.rs споживають рій і семантичний сервіс, жодного авторитету над
семантикою мови чи корпусом. Нещодавній AGENTS-комміт (65b85ee) додає
NLP-consumer інструкцію (lookup_concept.py usage) — UI стає консюмером
semantic-suggest sidecar.

## 3. Сильне
1. Активна розробка (24 комміти/тиждень) з чесними mobile-fix release-ами.
2. Swarm/oracle інтеграція як ПЕРШОКЛАСНИЙ UI до рою — унікальна ніша.
3. Три e2e-.mjs тести; benchmark script збережений.

## 4. Фронти / борги
| # | Пріоритет | Що |
|---|---|---|
| 1 | MED | registry-записи IDEA-CORE-CLJS-SPLIT-STATE та декілька gen0 (PLAYWRIGHT-SETUP, RELEASE-PS1, REAL-E2E) досі відкриті — звірити актуальність після v0.13.1 |
| 2 | LOW | benchmarks/*.my фікстури успадковані з my-lisp ери — синхронізувати або маркувати legacy |
| 3 | LOW | test_parse.rs у корені репо — прибрати в src-tauri/tests або видалити |

---
*Read-only.*
