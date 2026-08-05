# Language core · Ядро мови · Sprachkern

The language is independent from the IDE. Its first canonical Rust implementation now lives in `crates/my-idea-language`; the current ClojureScript evaluator remains an executable prototype during migration.

Мова незалежна від IDE. Її перша канонічна реалізація на Rust тепер міститься у `crates/my-idea-language`; під час міграції поточний інтерпретатор ClojureScript залишається виконуваним прототипом.

Die Sprache ist von der IDE unabhängig. Ihre erste kanonische Rust-Implementierung liegt jetzt in `crates/my-idea-language`; während der Migration bleibt der aktuelle ClojureScript-Interpreter ein ausführbarer Prototyp.

The current product direction is a DrRacket-like language environment: a definitions editor, program execution, an interactions/REPL area, readable diagnostics, and tools for exploring parsed forms. It serves our own language rather than implementing Racket itself. The UI and the language engine remain separate components.

Поточний напрям продукту — середовище мови на кшталт DrRacket: редактор визначень, запуск програми, область взаємодії/REPL, зрозумілі помилки та інструменти для дослідження розібраних форм. Воно обслуговує нашу власну мову, а не реалізує Racket. Інтерфейс і рушій мови залишаються окремими компонентами.

Die aktuelle Produktrichtung ist eine DrRacket-ähnliche Sprachumgebung: Definitionseditor, Programmausführung, Interaktions-/REPL-Bereich, verständliche Diagnosen und Werkzeuge zur Untersuchung geparster Formen. Sie dient unserer eigenen Sprache und implementiert nicht Racket selbst. Oberfläche und Sprachengine bleiben getrennte Komponenten.

## McCarthy foundation · Основа Маккарті · McCarthy-Grundlage

The first contract contains the seven elementary Lisp operations described by John McCarthy:

- `quote` returns data without evaluating it;
- `atom` recognizes atoms, including the empty list;
- `eq` compares two atoms;
- `car` returns the first element of a non-empty list;
- `cdr` returns the rest of a non-empty list;
- `cons` prepends an element to a list;
- `cond` evaluates clauses in order and selects the first true one.

`t` is the canonical true value. Invalid list operations produce explicit errors. Automated semantic tests are the compatibility contract for the future Rust core.

Перший контракт містить сім елементарних операцій Lisp: `quote`, `atom`, `eq`, `car`, `cdr`, `cons` і `cond`. Значення `t` є канонічною істиною. Некоректні операції зі списками повертають явні структуровані помилки, а автоматичні семантичні тести є контрактом сумісності.

Der erste Vertrag umfasst die sieben elementaren Lisp-Operationen `quote`, `atom`, `eq`, `car`, `cdr`, `cons` und `cond`. `t` ist der kanonische Wahrheitswert. Ungültige Listenoperationen liefern explizite strukturierte Fehler; automatisierte Semantiktests bilden den Kompatibilitätsvertrag.

The crate contains its own UTF-8 parser, value model, lexical environment frames, source spans, and structured errors. It has no Tauri dependency and no direct access to files, the network, or UI capabilities.

Крейт має власний UTF-8-парсер, модель значень, фрейми лексичного середовища, діапазони початкового коду та структуровані помилки. Він не залежить від Tauri й не має прямого доступу до файлів, мережі чи можливостей інтерфейсу.

Das Crate besitzt einen eigenen UTF-8-Parser, ein Wertmodell, lexikalische Umgebungsframes, Quellbereiche und strukturierte Fehler. Es hängt nicht von Tauri ab und hat keinen direkten Zugriff auf Dateien, Netzwerk oder UI-Funktionen.

## Migration path · Шлях міграції · Migrationspfad

1. Keep the primitive behavior specified by implementation-independent examples and tests.
2. Keep the independent Rust parser and value model covered by conformance tests.
3. Run the same conformance cases against ClojureScript and Rust during migration.
4. Expose Rust through Tauri on desktop and a portable boundary on web/mobile.
5. Remove the prototype only after the Rust core passes the full contract.

Наступний мовний крок — додати `lambda` та лексичні замикання поверх уже наявних дочірніх фреймів середовища, а потім провести однакові conformance-тести через реалізації ClojureScript і Rust.

Der nächste Sprachschritt ist `lambda` mit lexikalischen Closures auf Basis der vorhandenen untergeordneten Umgebungsframes. Danach werden dieselben Konformitätstests gegen die ClojureScript- und Rust-Implementierung ausgeführt.
