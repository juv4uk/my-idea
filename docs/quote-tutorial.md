# Remove the apostrophe · Приберіть апостроф · Entfernen Sie das Apostroph

## English

**my-lisp** is homoiconic: code and data share one notation. A leading `'` (the reader sugar for `quote`, see `docs/language-core.md`) tells the evaluator "treat this as inert data, don't run it." Delete that one character and the exact same text becomes a running program. This tutorial is a five-stage `.my` file: run each block with the `'`, read the printed result, then delete the `'` and run it again.

Try each pair with the CLI:

```powershell
cargo run --manifest-path crates/my-lisp-cli/Cargo.toml -- your-file.my
```

### Stage 1 — data vs. arithmetic

```lisp
'(+ 1 2 3)
```
Prints the inert list `(+ 1 2 3)` — three symbols, nothing computed.

```lisp
(+ 1 2 3)
```
Prints `6` — the exact same text, now executed.

### Stage 2 — data vs. a definition

```lisp
'(def answer 42)
```
Prints `(def answer 42)` — a description of a definition, not yet made.

```lisp
(def answer 42)
```
Prints `42` and `answer` now exists in the environment.

### Stage 3 — data vs. a function

```lisp
'(def square (lambda (x) (* x x)))
```
Prints the lambda expression itself, unevaluated.

```lisp
(def square (lambda (x) (* x x)))
(square 5)
```
Defines `square`, then prints `25`.

### Stage 4 — data vs. control flow

```lisp
'(cond ((eq 2 2) 100) (t 0))
```
Prints the `cond` form as a plain list.

```lisp
(cond ((eq 2 2) 100) (t 0))
```
Prints `100` — the condition actually runs.

### Stage 5 — growing the language itself

A `defmacro` receives its arguments as data (unevaluated, like `quote` does) and returns new code for the evaluator to run — this is how `lib/core.my` bootstraps derived forms without adding anything to the Rust core (see the bootstrap-boundary principle in `docs/language-core.md`):

```lisp
(defmacro unless (condition body)
    (list (quote cond)
        (list condition (quote ()))
        (list (quote t) body)))

(unless (eq 1 2) 'different)
```
Prints `different`. `unless` did not exist in the language a moment ago; you just grew it, out of the same quote/no-quote trick as stages 1–4.

## Українська

**my-lisp** гомоіконічна: код і дані записуються однаково. Провідний `'` (синтаксичний цукор для `quote`, див. `docs/language-core.md`) каже evaluator'у "став до цього як до інертних даних, не виконуй". Приберіть цей один символ — і той самий текст стає програмою, що виконується. Цей туторіал — `.my`-файл із п'яти етапів: запустіть кожен блок з `'`, подивіться на надрукований результат, потім приберіть `'` і запустіть знову.

Перевіряйте кожну пару через CLI:

```powershell
cargo run --manifest-path crates/my-lisp-cli/Cargo.toml -- your-file.my
```

### Етап 1 — дані проти арифметики

```lisp
'(+ 1 2 3)
```
Друкує інертний список `(+ 1 2 3)` — три символи, нічого не обчислено.

```lisp
(+ 1 2 3)
```
Друкує `6` — той самий текст, тепер виконаний.

### Етап 2 — дані проти визначення

```lisp
'(def answer 42)
```
Друкує `(def answer 42)` — опис визначення, ще не зроблений.

```lisp
(def answer 42)
```
Друкує `42`, і `answer` тепер існує в середовищі.

### Етап 3 — дані проти функції

```lisp
'(def square (lambda (x) (* x x)))
```
Друкує сам lambda-вираз, необчислений.

```lisp
(def square (lambda (x) (* x x)))
(square 5)
```
Визначає `square`, потім друкує `25`.

### Етап 4 — дані проти керуючої логіки

```lisp
'(cond ((eq 2 2) 100) (t 0))
```
Друкує форму `cond` як звичайний список.

```lisp
(cond ((eq 2 2) 100) (t 0))
```
Друкує `100` — умова справді виконується.

### Етап 5 — вирощування самої мови

`defmacro` отримує свої аргументи як дані (необчислені, як і при `quote`) і повертає новий код для виконання evaluator'ом — саме так `lib/core.my` розгортає похідні форми, не додаючи нічого до Rust-ядра (див. принцип межі bootstrap у `docs/language-core.md`):

```lisp
(defmacro unless (condition body)
    (list (quote cond)
        (list condition (quote ()))
        (list (quote t) body)))

(unless (eq 1 2) 'different)
```
Друкує `different`. Секунду тому `unless` не існувало в мові — ви щойно виростили його з того самого прийому quote/без-quote, що й у етапах 1–4.

## Deutsch

**my-lisp** ist homoikonisch: Code und Daten teilen sich eine Notation. Ein führendes `'` (der Reader-Zucker für `quote`, siehe `docs/language-core.md`) sagt dem Evaluator "behandle dies als reglose Daten, führe es nicht aus". Entfernen Sie dieses eine Zeichen, und derselbe Text wird zu einem laufenden Programm. Dieses Tutorial ist eine fünfstufige `.my`-Datei: Führen Sie jeden Block mit `'` aus, lesen Sie das ausgegebene Ergebnis, entfernen Sie dann das `'` und führen Sie ihn erneut aus.

Jedes Paar mit der CLI ausprobieren:

```powershell
cargo run --manifest-path crates/my-lisp-cli/Cargo.toml -- your-file.my
```

### Stufe 1 — Daten gegen Arithmetik

```lisp
'(+ 1 2 3)
```
Gibt die reglose Liste `(+ 1 2 3)` aus — drei Symbole, nichts berechnet.

```lisp
(+ 1 2 3)
```
Gibt `6` aus — derselbe Text, jetzt ausgeführt.

### Stufe 2 — Daten gegen eine Definition

```lisp
'(def answer 42)
```
Gibt `(def answer 42)` aus — eine Beschreibung einer Definition, noch nicht erstellt.

```lisp
(def answer 42)
```
Gibt `42` aus, und `answer` existiert nun in der Umgebung.

### Stufe 3 — Daten gegen eine Funktion

```lisp
'(def square (lambda (x) (* x x)))
```
Gibt den Lambda-Ausdruck selbst aus, unausgewertet.

```lisp
(def square (lambda (x) (* x x)))
(square 5)
```
Definiert `square`, gibt dann `25` aus.

### Stufe 4 — Daten gegen Kontrollfluss

```lisp
'(cond ((eq 2 2) 100) (t 0))
```
Gibt die `cond`-Form als einfache Liste aus.

```lisp
(cond ((eq 2 2) 100) (t 0))
```
Gibt `100` aus — die Bedingung wird tatsächlich ausgeführt.

### Stufe 5 — die Sprache selbst wachsen lassen

Ein `defmacro` erhält seine Argumente als Daten (unausgewertet, wie bei `quote`) und gibt neuen Code zurück, den der Evaluator ausführt — so bootstrappt `lib/core.my` abgeleitete Formen, ohne die Rust-Kern-Oberfläche zu vergrößern (siehe das Bootstrap-Grenze-Prinzip in `docs/language-core.md`):

```lisp
(defmacro unless (condition body)
    (list (quote cond)
        (list condition (quote ()))
        (list (quote t) body)))

(unless (eq 1 2) 'different)
```
Gibt `different` aus. Einen Moment zuvor existierte `unless` noch nicht in der Sprache — Sie haben es gerade aus demselben Quote/Ohne-Quote-Trick wie in den Stufen 1–4 wachsen lassen.
