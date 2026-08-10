# my-idea як System Observatory · Візуальний куратор екосистеми my-lisp

Записано 2026-08-08, авторське бачення користувача (juv4uk) для ролі `my-idea` у ширшій екосистемі: `my-lisp` (мова, вже виділена в [github.com/juv4uk/my-lisp](https://github.com/juv4uk/my-lisp)), компілятор (умовно `cml`) і майбутня апаратна Lisp-машина (умовно `fpga-lisp` — HDL-ядро для Sipeed Tang Primer 25K). Точні назви й статус цих двох репо ще не підтверджені остаточно. Цей документ — вихідний текст бачення як є, без редагування змісту.

## Позиція my-idea в системі

`my-idea` — четвертий компонент системи, але **не четверте джерело істини**. Його роль: **візуальний куратор стану `my-lisp → cml → fpga-lisp`**.

`my-idea` нічого не визначає. Він читає контракти, запускає перевірки, показує розбіжності й веде людину по причинно-наслідковому ланцюгу.

```
                   ┌──────────────────────┐
                   │       my-idea        │
                   │  VISUAL CURATOR      │
                   └──────────┬───────────┘
                              │ observe
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
      my-lisp                cml              fpga-lisp
   LANGUAGE TRUTH         COMPILER          HARDWARE TRUTH
        │                    │                   │
 conformance.my          compat/IR             ISA
 axioms                  codegen               RTL
 semantics               lowering           simulator
        └────────────────────┼───────────────────┘
                             │
                         evidence
```

`my-idea` має відповідати на просте питання: **«Чи є вся Lisp-система зараз узгодженою?»**

## Головний екран — System Map

Стартовий екран IDE:

```
┌─────────────────────────────────────────────────────────┐
│                   MY-LISP SYSTEM                       │
│                                                         │
│  LANGUAGE              COMPILER             MACHINE     │
│  my-lisp               cml                  fpga-lisp   │
│                                                         │
│  main                   master               main       │
│  l0.15                  c0.4                 ISA 0.4    │
│   ●                     ●                     ●         │
│   │                     │                     │         │
│   └──────────────►──────┴──────────────►──────┘         │
│                                                         │
│  Contract compatibility:        ✓                      │
│  Tier 1 Rust:                    23/23                  │
│  Tier 1 FPGA evaluator:          8/23                   │
│  Tier 1 CML compiled:            6/23                   │
│  Last ecosystem check:           ✓                     │
└─────────────────────────────────────────────────────────┘
```

Один погляд на весь проєкт.

## Не просто зелені лампочки — межа можливостей

Найцінніше — показувати, де саме проходить межа можливостей:

```
LANGUAGE FEATURE       Rust     FPGA eval     CML→FPGA

quote                    ✓          ✓            ✓
atom                     ✓          ✓            ✓
eq                       ✓          ✓            ✓
car                      ✓          ✓            ✓
cdr                      ✓          ✓            ✓
cons                     ✓          ✓            ✓
cond                     ✓          ✓            ✓

lambda                   ✓          ✓            △
variadic lambda          ✓          ✗            ✗
exact integer            ✓          △            ✗
exact rational            ✓          ✗            ✗
inexact real              ✓          ✗            ✗
```

Жива карта розвитку Lisp-машини. Не roadmap, написаний вручну, а карта, згенерована з реальних тестів.

## Найкраща одиниця UI — fixture

Оскільки `conformance.my` уже стає семантичним центром, fixture — базовий об'єкт `my-idea`. Клік на:

```
(eq 3 3.0)
```

відкриває:

```
Fixture #N

Tier:       2
Axiom:      S1
Expected:   ()

────────────────────────────

Rust reference
✓ ()

FPGA evaluator
✗ unsupported numeric exactness

CML
✗ backend cannot encode InexactReal

────────────────────────────

Language:
my-lisp@fef422b

Compiler:
cml@...

Machine:
fpga-lisp@...

────────────────────────────

Blocking path:
CML parser
   ↓
numeric IR
   ↓
FPGA tag missing
```

Це не IDE в традиційному сенсі — це **semantic debugger усієї екосистеми**.

## Граф проходження виразу

Наприклад, для:

```
((lambda (x) (cons x '(b c))) 'a)
```

`my-idea` показує:

```
SOURCE
  │
  ▼
my-lisp parser
  │
  ▼
S-expression
  │
  ├──────────────────┐
  ▼                  ▼
Rust eval            CML
  │                  │
(a b c)              IR
                     │
                     ▼
                    ASM
                     │
                     ▼
                 assembler
                     │
                     ▼
                    BIN
                     │
                     ▼
                   FPGA
                     │
                     ▼
                  (a b c)
```

Якщо щось розійшлося:

```
Rust → (a b c)
FPGA → (a c)

          ↑
     mismatch here
```

Клік — assembly та register trace. Надзвичайно корисно для розробки `cml`.

## Три типи контрактів

Не код репозиторіїв, а саме інтерфейси, які `my-idea` лише складає разом:

**Language Contract** — з `my-lisp`: `conformance.my`, axioms, tiers, language version.

**Compiler compatibility** — з `cml`: supported language contract, supported ISA version, supported forms.

**Machine Contract** — з `fpga-lisp`: ISA, tags, register ABI, opcodes, machine capabilities.

## Дуже важливо: не hardcode у UI

Не писати в Svelte:

```
const supportsLambda = true;
```

Ніколи. `my-idea` має будувати інтерфейс із машинно-читаних manifests:

```
my-lisp:
  ecosystem.my

cml:
  compatibility.my

fpga-lisp:
  machine.my
```

UI лише візуалізує.

## Поняття Evidence

Центральна ідея `my-idea`: кожна зелена галочка повинна мати доказ. Не:

```
CONS ✓
```

а:

```
CONS ✓

Evidence:
  fixture: tier1-cons-01
  my-lisp: abc123
  cml:     def456
  fpga:    789abc
  simulator: passed
```

Клік на ✓ відкриває, чому це вважається істинним. Стикується з provenance-філософією самого `my-lisp`: `my-idea` застосовує provenance до процесу розробки.

## Evidence Graph

```
             G3 CODE = DATA
                  │
         ┌────────┴────────┐
         │                 │
      quote fixture      list fixture
         │                 │
     ┌───┼────┐        ┌───┼────┐
     ▼   ▼    ▼        ▼   ▼    ▼
   Rust FPGA CML     Rust FPGA CML
```

```
G1  █████████████ 100%
G2  ███████████░░  86%
G3  ███████░░░░░░  58%
G4  ████░░░░░░░░░  31%
S1  █████░░░░░░░░  42%
```

Не фальшиві відсотки зрілості — краще факти:

```
G3:
  7 constitutive fixtures
  Rust: 7 passed
  FPGA evaluator: 5 passed
  CML path: 3 passed
```

Факти замість рейтингу.

## Compatibility Lens

```
[ my-lisp main ]
[ cml master ]
[ fpga-lisp main ]
```

```
⚠ Compatibility mismatch

cml expects:
  ISA 0.4

fpga-lisp provides:
  ISA 0.5

Breaking changes:
  CALL encoding
  closure tag

Affected fixtures:
  lambda-basic
  nested-application
  recursive-call
```

Візуальне вирішення питання синхронізації.

## Timeline

```
Aug 09

10:20 my-lisp
      exactness becomes value property

10:44 cml
      ⚠ compatibility stale

11:15 fpga-lisp
      new INEXACT tag

11:27 cml
      backend updated

11:31 ecosystem
      ✓ 17 fixtures pass on all paths
```

Не просто Git history трьох репо, а історія системи як одного організму.

## Decisions

Панель для принципових рішень:

```
NO set!
NO first-class continuations
persistent structures
exactness is semantic
C-core dropped
Rust + FPGA
```

```
ARCHITECTURAL DECISIONS

✓ Exactness is a value property
  commit: ...
  affects: my-lisp, cml, fpga-lisp

✓ No general set!
  affects: language only

✓ C implementation dropped
  affects: ecosystem architecture
```

Візуальна пам'ять проєкту.

## Технічна реалізація

`my-idea` уже на Tauri/Svelte — backend aggregator на Rust:

```
src-tauri/
  ecosystem/
    git.rs
    my_lisp.rs
    cml.rs
    fpga.rs
    compatibility.rs
    evidence.rs
```

Rust backend:
- знаходить локальні clones;
- читає manifests;
- викликає `git rev-parse`;
- запускає tests;
- запускає CML;
- assembler;
- simulator;
- збирає структурований результат.

Svelte лише показує його.

## Не починати з GitHub API

Спочатку `my-idea` повинен працювати повністю локально:

```
~/projects/
  my-lisp/
  cml/
  fpga-lisp/
```

Відповідає offline-first філософії. GitHub потім може бути лише: remote status, new commits, CI results, release information — але не джерелом основного стану.

## MVP

Не будувати одразу «центр керування космічним кораблем». Перша версія Visual Curator:

1. знаходить три локальні repo;
2. показує branch + SHA;
3. читає language contract version / ISA version / CML compatibility;
4. показує compatibility;
5. кнопка Run ecosystem check;
6. виводить три колонки:

```
Fixture        Rust      FPGA       CML
quote           ✓         ✓          ✓
atom            ✓         ✓          ✓
eq              ✓         ✓          ✓
car             ✓         ✓          ✗
```

Все. Це вже буде неймовірно корисно.

## Наступний етап — інтеграція з редактором

На другому етапі — об'єднання з редактором `my-idea`. Пишеш Lisp:

```
(unify '(parent (var x) bob)
       '(parent alice bob)
       '())
```

і поруч не просто Run:

```
Run with:

● Rust evaluator
○ FPGA evaluator
○ CML → FPGA
○ ALL
```

Натискаєш ALL:

```
Rust        → ((x . alice))  ✓
FPGA eval   → ((x . alice))  ✓
CML/FPGA    → ((x . alice))  ✓

SEMANTIC AGREEMENT
```

Кінцева форма `my-idea`.

## Чому "System Observatory"

Це вже не просто редактор для Lisp. Це місце, де видно шлях однієї ідеї від S-expression через компілятор до фізичних вентилів FPGA — і назад до результату.

Тому цей модуль варто називати не "Project Manager", а, наприклад, **System Observatory** або українською **«Обсерваторія»**.

`my-lisp` визначає сенс, `cml` переводить його, `fpga-lisp` втілює, а `my-idea` спостерігає, чи вони досі говорять про одне й те саме.
