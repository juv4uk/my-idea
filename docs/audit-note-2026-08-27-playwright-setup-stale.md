# Аудит-нотатка: IDEA-PLAYWRIGHT-BROWSER-SETUP позначена відкритою, хоча вирішена

**Статус:** ЗВІТ (виявлено, не виправлено).
**Джерело:** agent-team аудит суперечностей/застарілих пунктів, повний
звіт — `/home/agents/ecosystem/docs/agent-team-contradiction-audit-2026-08-27.md`
(§5.2).

## Знахідка

`tasks.my:56-60`, задача `IDEA-PLAYWRIGHT-BROWSER-SETUP`, досі
`(done . ())`.

Вирішено комітом `a865ed5` (2026-08-22): `AGENTS.md:277-280` документує
`bunx playwright install` як one-time setup, "verified end-to-end (build
EXIT=0, npm test 37/37 PASS)". CI (`.github/workflows/ci.yml:40`) вже
запускає `bunx playwright install chromium --with-deps`.

Ця застарілість вже раз була помічена в самому репо
(`docs/arch-recovery-review-2026-08-25.md:38`: "PLAYWRIGHT-SETUP ...
досі відкриті — звірити актуальність"), але після того запису
`tasks.my` теж не оновили.

## Виправлення (не зроблено)

Позначити `IDEA-PLAYWRIGHT-BROWSER-SETUP` як `(done . t)` з посиланням
на коміт `a865ed5` і CI-крок як доказ.
