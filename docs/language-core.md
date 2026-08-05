# my-lisp language core · Ядро мови my-lisp · my-lisp-Sprachkern

> **A small language that grows itself. · Маленька мова, що вирощує себе. · Eine kleine Sprache, die sich selbst wachsen lässt.**

The language is named **my-lisp** and is independent from the IDE. Its first canonical Rust implementation lives in `crates/my-lisp`; the current ClojureScript evaluator remains an executable prototype during migration.

Мова має назву **my-lisp** і є незалежною від IDE. Її перша канонічна реалізація на Rust міститься у `crates/my-lisp`; під час міграції поточний інтерпретатор ClojureScript залишається виконуваним прототипом.

Die Sprache heißt **my-lisp** und ist von der IDE unabhängig. Ihre erste kanonische Rust-Implementierung liegt in `crates/my-lisp`; während der Migration bleibt der aktuelle ClojureScript-Interpreter ein ausführbarer Prototyp.

The canonical source-file extension is **`.my`** (for example, `welcome.my`). The generic `.lisp` extension remains a compatible alias.

Канонічне розширення файлів початкового коду — **`.my`** (наприклад, `welcome.my`). Загальне розширення `.lisp` залишається сумісним псевдонімом.

Die kanonische Dateiendung für Quellcode ist **`.my`** (zum Beispiel `welcome.my`). Die allgemeine Endung `.lisp` bleibt ein kompatibler Alias.

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

Exact rational arithmetic is a kernel mechanism: `/` accepts exact integers and rational values, reduces every fraction, and prints results such as `5/336` without floating-point rounding.

Крейт має власний UTF-8-парсер, модель значень, фрейми лексичного середовища, діапазони початкового коду та структуровані помилки. Він не залежить від Tauri й не має прямого доступу до файлів, мережі чи можливостей інтерфейсу.

Точна раціональна арифметика є механізмом ядра: `/` приймає точні цілі та раціональні значення, скорочує кожен дріб і виводить результати на кшталт `5/336` без floating-point округлення.

Das Crate besitzt einen eigenen UTF-8-Parser, ein Wertmodell, lexikalische Umgebungsframes, Quellbereiche und strukturierte Fehler. Es hängt nicht von Tauri ab und hat keinen direkten Zugriff auf Dateien, Netzwerk oder UI-Funktionen.

Exakte rationale Arithmetik ist ein Kernmechanismus: `/` akzeptiert exakte Ganzzahlen und rationale Werte, kürzt jeden Bruch und gibt Ergebnisse wie `5/336` ohne Gleitkommarundung aus.

## Bootstrap boundary · Межа саморозгортання · Bootstrap-Grenze

Rust provides only the mechanisms it implements particularly well: memory-safe runtime values, UTF-8 reading, lexical closures, deterministic evaluation, stack control, structured diagnostics, and an explicit capability boundary. Higher-level language features and the standard library should be written in the small Lisp itself whenever the existing core can express them.

Rust надає лише ті механізми, які він виконує особливо добре: безпечні щодо пам’яті значення, читання UTF-8, лексичні замикання, детерміноване обчислення, контроль стека, структуровану діагностику та явну межу системних можливостей. Високорівневі можливості й стандартну бібліотеку слід писати самою маленькою Lisp-мовою щоразу, коли наявне ядро вже може їх виразити.

Rust stellt nur die Mechanismen bereit, die es besonders gut umsetzt: speichersichere Laufzeitwerte, UTF-8-Lesen, lexikalische Closures, deterministische Auswertung, Stack-Kontrolle, strukturierte Diagnosen und eine explizite Capability-Grenze. Höhere Sprachfunktionen und die Standardbibliothek sollen in der kleinen Lisp-Sprache selbst geschrieben werden, sobald der vorhandene Kern sie ausdrücken kann.

`lambda` belongs to the Rust semantic kernel because it makes user-defined functions and self-hosted libraries possible. Derived forms such as `defn`, list helpers, logical combinators, and teaching examples belong in a bootstrapped Lisp library rather than as Rust built-ins.

The first bootstrapped library is `lib/core.my`. Its definitions (`identity`, `not`, `pair`, `second`, `third`, `caar`, and `cadr`) are ordinary my-lisp code. Rust supplies only `def` and lexical binding as the mechanism needed to load persistent named definitions.

`lambda` належить до семантичного Rust-ядра, бо робить можливими користувацькі функції та саморозгорнуті бібліотеки. Похідні форми на кшталт `defn`, допоміжні функції списків, логічні комбінатори й навчальні приклади мають жити у bootstrap-бібліотеці Lisp, а не бути вбудованими у Rust.

Перша bootstrap-бібліотека — `lib/core.my`. Її визначення (`identity`, `not`, `pair`, `second`, `third`, `caar` і `cadr`) є звичайним кодом my-lisp. Rust надає лише `def` і лексичне зв’язування як механізм завантаження постійних іменованих визначень.

`lambda` gehört zum semantischen Rust-Kern, weil es benutzerdefinierte Funktionen und selbst gehostete Bibliotheken ermöglicht. Abgeleitete Formen wie `defn`, Listenhilfen, logische Kombinatoren und Lernbeispiele gehören in eine gebootstrappte Lisp-Bibliothek statt in Rust-Built-ins.

Die erste Bootstrap-Bibliothek ist `lib/core.my`. Ihre Definitionen (`identity`, `not`, `pair`, `second`, `third`, `caar` und `cadr`) sind gewöhnlicher my-lisp-Code. Rust stellt nur `def` und lexikalische Bindung als Mechanismus zum Laden dauerhafter benannter Definitionen bereit.

## Migration path · Шлях міграції · Migrationspfad

1. Keep the primitive behavior specified by implementation-independent examples and tests.
2. Keep the independent Rust parser and value model covered by conformance tests.
3. Run the same conformance cases against ClojureScript and Rust during migration.
4. Expose Rust through Tauri on desktop and a portable boundary on web/mobile.
5. Remove the prototype only after the Rust core passes the full contract.

Наступний мовний крок — додати `lambda` та лексичні замикання поверх уже наявних дочірніх фреймів середовища, а потім провести однакові conformance-тести через реалізації ClojureScript і Rust.

Der nächste Sprachschritt ist `lambda` mit lexikalischen Closures auf Basis der vorhandenen untergeordneten Umgebungsframes. Danach werden dieselben Konformitätstests gegen die ClojureScript- und Rust-Implementierung ausgeführt.
