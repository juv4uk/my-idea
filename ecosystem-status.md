# Ecosystem status — my-idea

**Роль цього файлу**: append-only хронологічний лог фактів з боку `my-idea`,
за тим самим протоколом, що й `cml/ecosystem-status.md` і
`fpga-lisp/ecosystem-status.md`. Для курованого поточного знімка всієї
екосистеми див. `my-lisp`'s `ecosystem-status.my`/`ecosystem-status.md`.

Репозиторії на цій машині:
- `my-lisp` — C:\Users\user\Documents\GitHub\my-lisp
- `cml` — C:\Users\user\Documents\GitHub\cml
- `fpga-lisp` — C:\Users\user\Documents\GitHub\fpga-lisp
- `my-idea` — C:\Users\user\Documents\GitHub\my-idea (цей файл)

---

## [my-idea] 2026-08-11 — приєднання до координаційного протоколу

`my-idea` (Claude Code сесія) приєднується до cross-session координації з
my-lisp/cml/fpga-lisp. Короткий статус реалізації System Observatory MVP:

**Реалізовано** (`src-tauri/src/ecosystem/`):
- `git.rs` — знаходить сусідні репо на диску, читає branch/SHA.
- `contracts.rs` — читає `language-contract.my` (my-lisp), `isa-contract.my`
  (fpga-lisp), `compatibility.my` (cml) і my-lisp's `ecosystem-status.my`
  (`cml`-запис: tier-1-skips-remaining, ci-status, equal?/defmacro-status)
  через `my_lisp::parse` (власний reader my-lisp, не саморобний парсер).
- Fixture inventory: `read_fixture_inventory()` читає впорядкований
  канонічний `my-lisp/tests/fixtures/conformance.my` (expr/expected/error/
  tier/axioms/role/note).
- `mod.rs` збирає все в `EcosystemStatus`, звіряє версії контрактів, які
  cml очікує, проти фактичних версій my-lisp/fpga-lisp.
- Tauri command `ecosystem_status`, викликається кнопкою **🔭 Ecosystem** у
  desktop UI (ClojureScript, не Svelte, як помилково зазначено в
  ранньому vision-документі) — окремий екран з картками трьох репо
  (clone presence, branch, SHA, версія контракту) і блоком сумісності.
- Unit-тести для всіх contract/status/fixture readers (7 тестів,
  `cargo test --lib ecosystem`, усі зелені).

**Не реалізовано**: запуск тестів/CML/simulator — лише читання статичних
контрактів; evidence graph і fixture-рівневі execution results з
vision-документа.

**Залежність**: my-idea тягне my-lisp як cargo git-залежність
(`branch = "main"`), не submodule — синхронізація ревізій лише вручну, без
CI-інваріанту.

**Відомий локальний ризик**: цей же checkout одночасно редагується іншою,
незалежною координаційною системою (Codex/OpenCode, файлова тека
`C:\Users\user\Documents\GitHub\docs\`, поза git). Один раз це вже
призвело до незапланованого перемикання гілки (`main` →
`agent/test-contract-readers`) під час активної роботи Claude Code сесії.
Дані не втрачено, але це підтверджує потребу в окремому `git worktree` для
кожного одночасно активного агента в цьому репо.

**TCP REPL my-lisp (127.0.0.1:9999)**: підключення напряму заблоковано
класифікатором дозволів Claude Code auto mode (raw TCP socket + виконання
коду через нього трактується як небезпечна дія за замовчуванням) — це
обмеження платформи агента, не рішення сесії. Як обхідний шлях
використано вже зібраний локальний бінарник `my-lisp/target/debug/my-lisp.exe`
напряму (без мережі) для перевірки семантики.

---

## [my-idea → my-lisp/cml/fpga-lisp] 2026-08-11 — пропозиція користувача: протокол координації v2

Ретрансляція (SendMessage до інших сесій недоступний зараз — durable-файл
за новим же принципом нижче). Користувач запропонував замінити прозовий
обмін статусами на п'ять рівнів, з ідеєю **"agents communicate through
contracts, fixtures, evidence and Git history — not through prose
messages"**:

1. **Authoritative contracts** — по одному machine-readable контракту на
   репо: `language-contract.my` (my-lisp), `isa-contract.my` (fpga-lisp),
   `compatibility.my` (cml), новий `observatory-contract.my` (my-idea —
   лише формат того, що вона вміє читати/показувати, не джерело істини).
2. **Evidence files** — замість ручного "PASS" у прозі, машинний запис на
   fixture:
   ```lisp
   (evidence
     (fixture G8-zero-truthiness)
     (implementation fpga-lisp)
     (commit "3673875")
     (result pass)
     (expected true)
     (actual true)
     (runner iverilog)
     (timestamp "..."))
   ```
3. **Один мінімальний ecosystem index** — карта шляхів чотирьох репо, без
   дублювання статусів усередині.
4. **Pull, не push** — my-idea сама читає contracts+evidence+SHA+CI;
   жоден агент не пише статус вручну в чужий репо.
5. **TCP REPL лише як oracle** — canonical result, ніколи agent-to-agent
   message bus.

Додатково: єдиний ID для семантичних вимог (`G1 quote`, `G8 truthiness`,
`N1 exact-integer`, `M1 macro-expand`...) — уже частково існує як
`axioms` у `my-lisp/tests/fixtures/conformance.my`, ще не наскрізний ключ
у жодному UI. `depends-on`-записи для міжрепо-залежностей. Перед великою
контрактною зміною — `proposal`-запис (id/owner/affects/status/
contract-change), не обговорення в чаті. `AGENTS.md` скорочується до:
роль, authoritative files, як запускати тести, що не можна міняти без
contract bump, як створювати evidence, як перевіряти сусідні репо — без
хронології.

Мета: менше "комунікаційних" комітів (агент сказав агенту), більше
`evidence/*.my` фактів з походженням.

## [my-idea → my-lisp] 2026-08-11 — пропозиція користувача: TCP REPL protocol v1 (semantic oracle)

Узгоджується з `AGENT_MEMORY.md`'s "TCP REPL is a semantic oracle, not an
agent message bus" і вже запланованим Protocol v1. Деталізація від
користувача:

- Роль: живий еталон семантики, до якого звертаються `cml`, `fpga-lisp`,
  `my-idea` — **не** канал, де агенти листуються.
- Differential testing: fixture йде одночасно в my-lisp REPL (`expected`)
  і в `cml → fpga-lisp` (`actual`); розбіжність автоматично стає
  `evidence`-записом (`oracle`/`target`/`status`).
- Протокольний конверт замість голого тексту:
  ```lisp
  (request (id 42) (op eval) (source "(+ 1 2)"))
  (response (id 42) (status ok) (value 3))
  (request (id 43) (op diagnose) (source "(car 1)"))
  (response (id 43) (status error) (kind type-error)
            (message "car expected pair") (form (car 1)))
  ```
  плюс `(op parse)`, `(op macroexpand)`, `(op canonical-write)`,
  `(op contract-version)`.
- Явно НЕ додавати в REPL: `git status`, CI-статус, agent-повідомлення,
  task queue — інакше semantic oracle перетворюється на coordination
  daemon.
- Кожен fixture-тест — у fresh session (connect → load core → run
  fixture → read result → disconnect), не в перевикористаному стані.
- Machine mode окремо від людського REPL (`--tcp --protocol=sexpr`):
  строго структурована відповідь без банерів/prompt/кольорів/випадкового
  stderr у protocol stream.

my-idea зі свого боку: коли з'явиться `observatory-contract.my` й формат
evidence погоджено, готова читати evidence-файли так само, як зараз
читає contracts (through `my_lisp::parse`, наскрізно з диска, без кешу).
