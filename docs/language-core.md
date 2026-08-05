# Language core · Ядро мови · Sprachkern

The language is intended to become independent from the IDE. Its canonical implementation will be a Rust library; the current ClojureScript evaluator is an executable prototype of the semantics.

Мова має стати незалежною від IDE. Її канонічною реалізацією буде бібліотека на Rust; поточний інтерпретатор ClojureScript є виконуваним прототипом семантики.

Die Sprache soll von der IDE unabhängig werden. Ihre kanonische Implementierung wird eine Rust-Bibliothek; der aktuelle ClojureScript-Interpreter ist ein ausführbarer Prototyp der Semantik.

The current product direction is a DrRacket-like language environment: a definitions editor, program execution, an interactions/REPL area, readable diagnostics, and tools for exploring parsed forms. It serves our own language rather than implementing Racket itself. The UI and the language engine remain separate components.

Поточний напрям продукту — середовище мови на кшталт DrRacket: редактор визначень, запуск програми, область взаємодії/REPL, зрозумілі помилки та інструменти для дослідження розібраних форм. Воно обслуговує нашу власну мову, а не реалізує Racket. Інтерфейс і рушій мови залишаються окремими компонентами.

Die aktuelle Produktrichtung ist eine DrRacket-ähnliche Sprachumgebung: Definitionseditor, Programmausführung, Interaktions-/REPL-Bereich, verständliche Diagnosen und Werkzeuge zur Untersuchung geparster Formen. Sie dient unserer eigenen Sprache und implementiert nicht Racket selbst. Oberfläche und Sprachengine bleiben getrennte Komponenten.

## McCarthy foundation

The first contract contains the seven elementary Lisp operations described by John McCarthy:

- `quote` returns data without evaluating it;
- `atom` recognizes atoms, including the empty list;
- `eq` compares two atoms;
- `car` returns the first element of a non-empty list;
- `cdr` returns the rest of a non-empty list;
- `cons` prepends an element to a list;
- `cond` evaluates clauses in order and selects the first true one.

`t` is the canonical true value. Invalid list operations produce explicit errors. Automated semantic tests are the compatibility contract for the future Rust core.

## Migration path

1. Keep the primitive behavior specified by implementation-independent examples and tests.
2. Introduce an independent parser and value model in a Rust crate.
3. Run the same conformance cases against ClojureScript and Rust during migration.
4. Expose Rust through Tauri on desktop and a portable boundary on web/mobile.
5. Remove the prototype only after the Rust core passes the full contract.
