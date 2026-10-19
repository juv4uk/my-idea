# my-idea-tui

Термінальна (TUI) поверхня **my-idea** на **Ratatui** — *той самий продукт*, що GUI,
лише в терміналі. Це **не** новий рушій, **не** друга мова і **не** другий продукт.

Відповідає задачі [#31](https://github.com/juv4uk/my-idea/issues/31) і результату
дослідження [#84](https://github.com/juv4uk/my-idea/issues/84). Продуктова межа —
`docs/ADR-003-SIMPLE-SELF-BUILDING-IDE.md`.

> **Статус: збірка й тести Rust пройшли у GitHub Actions (run #39260079011); реальний SENS file-mode smoke ще не підтверджений.**
> Збірку Ratatui перевірено на GitHub-hosted runner; SENS-рушій окремо має запускатися із pinned gitlink.
> Це каркас для огляду й першого зрізу, а не «готово».

## Перша вертикаль (#31)

`Open workspace → Explorer → Edit → Save → Run → Output → Stop`

- **Explorer** — дерево/список файлів робочого каталогу, активний файл.
- **Editor** — відкрити → редагувати → зберегти через спільний із GUI workspace-save механізм.
- **Console/Output** — введення, історія (↑/↓), прокрутка; вивід рушія з точним
  походженням **stdout / stderr / system**.
- **Run** — запускає збережений файл через `sens <file>` або сумісний `my-lisp <file>`, не stdin REPL; **Stop** — реальна зупинка процесу.

## Семантика й піни

- Єдине джерело — **поточний pinned `external/sens`**. TUI лише запускає той самий
  `my-lisp` CLI, який збирається з цього gitlink (як і GUI-консоль).
- **Gitlink `external/sens` не змінюється** для додавання TUI; #81 — окрема задача.
- **Жодного власного Lisp-інтерпретатора** в TUI. Жодної другої семантики.
- Старі згадки `my-lisp`/legacy — не нормативна підстава повертати стару семантику.

## Залежності

```toml
ratatui   = { version = "0.30", default-features = false, features = ["crossterm"] }
crossterm = "0.29"
```

Один термінальний бекенд, без зайвих шарів. Рушій **не** додається як Cargo-залежність.

## Збірка й запуск

```sh
# 1) ініціалізувати саме закріплений підмодуль:
git submodule update --init external/sens
# 2) зібрати CLI з поточного exact gitlink:
cargo build --release --manifest-path external/sens/Cargo.toml -p sens-cli --bin my-lisp
# 3) окремо зібрати TUI (не чіпає кореневий lockfile):
cargo build --release --manifest-path my-idea-tui/Cargo.toml
# 4) запустити з будь-якого поточного каталогу:
./my-idea-tui/target/release/my-idea-tui
```

Якщо CLI не зібрано в `external/sens/target/{release,debug}`, консоль показує інформативну помилку. `PATH` та `MY_LISP_BIN` не підміняють pinned джерело. Шлях до вже зібраного бінарника сам собою не є SHA-attestation: після зміни gitlink перебудуйте CLI.

## Клавіші

| Клавіша | Дія |
|---|---|
| `Tab` / `Shift-Tab` | перемикання панелей |
| `Enter` | Explorer: відкрити; Console: надіслати; Editor: новий рядок |
| `↑` / `↓` | Explorer: вибір; Console: історія введених команд |
| `PageUp` / `PageDown` / `End` | Console: попередній вивід / наступний / найновіший |
| `F5` | Запуск збереженого файла через pinned SENS CLI |
| `Backspace` (Explorer) | до батьківського каталогу в межах workspace |
| `F9` | Stop (файл і REPL) |
| `Ctrl-S` | зберегти поточний файл |
| `Ctrl-C` / `Ctrl-Q` | вихід (при незбережених змінах потрібно повторити) |
| `Esc` | повернутися до Explorer |

Мета-команди консолі: `:мова ук|de|en`, `:run`, `:stop`, `:save`.

## Мова інтерфейсу

`uk → de → en` — за чинними ключами GUI (`src-cljs/my_idea/i18n.cljs`).
Мова інтерфейсу **не** є перемикачем доменної/семантичної SENS-проєкції.

## Що далі (поза першою вертикалею)

AST/Preview, вкладки, палітра команд, повний паритет панелей — після працездатної
вертикалі. Візуальні компоненти ClojureScript і Ratatui **окремі**; повторно
використовується контракт команд і семантика виконання, а не віджети.

## Точне виконання

- F5 не надсилає файл у REPL. Перед запуском незбережені зміни треба зберегти через Ctrl+S.
- Текстовий `.lisp` читається редактором; фізичний `.sens` залишається бінарним артефактом і не відображається як UTF-8 Lisp.
- Інтерактивний REPL запускається окремо при першій введеній команді. Run показує stdout/stderr та код завершення CLI у консолі.
- Фактична збірка Ratatui підтверджена [GitHub Actions #39260079011](https://github.com/juv4uk/my-idea/actions/runs/39260079011); це ще **не** доказ end-to-end семантики або provenance бінарника SENS.
