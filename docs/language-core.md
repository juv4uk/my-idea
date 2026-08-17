# my-lisp language core · Ядро мови my-lisp · my-lisp-Sprachkern

> **A small language that grows itself. · Маленька мова, що вирощує себе. · Eine kleine Sprache, die sich selbst wachsen lässt.**

The language is named **my-lisp** and is independent from the IDE. Its canonical implementation lives in `crates/my-lisp` (Rust), which powers both the desktop shell and the Web build via WebAssembly. The initial ClojureScript prototype has been fully replaced.

Мова має назву **my-lisp** і є незалежною від IDE. Її канонічна реалізація міститься у `crates/my-lisp` (Rust), яка забезпечує роботу як десктопної оболонки, так і веб-збірки через WebAssembly. Початковий прототип на ClojureScript повністю замінено.

Die Sprache heißt **my-lisp** und ist von der IDE unabhängig. Ihre kanonische Implementierung liegt in `crates/my-lisp` (Rust), welche sowohl die Desktop-Hülle als auch den Web-Build über WebAssembly antreibt. Der anfängliche ClojureScript-Prototyp wurde vollständig ersetzt.

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

## Available Operations and Forms · Доступні операції та форми · Verfügbare Operationen und Formen

### Built-in Forms (Rust core) · Вбудовані форми · Eingebaute Formen
- `quote` (`'`) — Returns the expression unevaluated · Повертає вираз без обчислення · Gibt den Ausdruck unberechnet zurück
- `lambda` — Creates an anonymous function (closure) · Створює анонімну функцію (замикання) · Erstellt eine anonyme Funktion (Closure)
- `def` — Defines a variable or function in the current environment · Визначає змінну або функцію в поточному середовищі · Definiert eine Variable oder Funktion in der aktuellen Umgebung
- `defmacro` — Defines a macro (compile-time expansion) · Створює макрос (розкривається на етапі оцінки) · Definiert ein Makro (Erweiterung zur Kompilierzeit)
- `list` — Creates a list from evaluated arguments · Створює список з обчислених аргументів · Erstellt eine Liste aus ausgewerteten Argumenten
- `cond` — Conditional logic · Умовна логіка · Bedingte Logik
- `atom` — Checks if a value is an atom (not a list/pair) · Перевіряє, чи є значення атомом (не списком/парою) · Prüft, ob ein Wert ein Atom ist (keine Liste/Paar)
- `eq` — Checks equality between two atoms · Перевіряє рівність двох атомів · Prüft die Gleichheit zwischen zwei Atomen
- `car` — Returns the first element of a list/pair · Повертає перший елемент списку/пари · Gibt das erste Element einer Liste/eines Paares zurück
- `cdr` — Returns the tail of a list/pair · Повертає хвіст списку/пари · Gibt den Rest einer Liste/eines Paares zurück
- `cons` — Constructs a pair or adds an element to a list · Створює пару або додає елемент до списку · Erstellt ein Paar oder fügt ein Element zu einer Liste hinzu
- `+`, `-`, `*`, `/` — Arithmetic operations. `/` produces exact rational fractions (e.g. `1/3`) for integers. · Арифметичні операції. `/` для цілих чисел створює точні раціональні дроби (напр. `1/3`). · Arithmetische Operationen. `/` erzeugt für Ganzzahlen exakte rationale Brüche (z. B. `1/3`).

### Standard Library (`lib/core.my`) · Стандартна бібліотека · Standardbibliothek
- `identity` — Returns its argument · Повертає переданий аргумент · Gibt sein Argument zurück
- `not` — Logical NOT · Логічне заперечення · Logisches NICHT
- `pair` — Creates a list of two elements · Створює список із двох елементів · Erstellt eine Liste aus zwei Elementen
- `second` — Returns the second element of a list · Повертає другий елемент списку · Gibt das zweite Element einer Liste zurück
- `third` — Returns the third element of a list · Повертає третій елемент списку · Gibt das dritte Element einer Liste zurück
- `caar` — `(car (car x))`
- `cadr` — `(car (cdr x))`

## Unified execution · Уніфіковане виконання · Einheitliche Ausführung

The migration from the ClojureScript prototype to the Rust core is complete. The Rust implementation conforms to the primitive behavior specified by implementation-independent examples and tests.

Міграція з прототипу на ClojureScript до Rust-ядра завершена. Rust-реалізація відповідає примітивній поведінці, визначеній незалежними від реалізації прикладами та тестами.

Die Migration vom ClojureScript-Prototyp zum Rust-Kern ist abgeschlossen. Die Rust-Implementierung entspricht dem primitiven Verhalten, das durch implementierungsunabhängige Beispiele und Tests spezifiziert ist.

The desktop and mobile Tauri shell exposes `evaluate_my_lisp`, while the Web/PWA build uses `crates/my-lisp-wasm` to run the canonical Rust engine directly in the browser via WebAssembly. Both host boundaries utilize single-pass parsing (`eval_parsed_expressions`), parsing editor source code once to produce both the AST view and evaluation result. If WebAssembly fails to load in a web environment, the application gracefully degrades by displaying a clear UI error instead of an infinite loading state.

Оболонка Tauri для desktop і mobile надає `evaluate_my_lisp`, а Web/PWA-збірка використовує `crates/my-lisp-wasm` для запуску канонічного Rust-рушія безпосередньо у браузері через WebAssembly. Обидві межі використовують однопрохідний парсинг (`eval_parsed_expressions`), аналізуючи код редактора один раз як для відображення AST, так і для обчислення. Якщо WebAssembly не завантажується у веб-середовищі, застосунок витончено деградує, відображаючи чітку помилку інтерфейсу замість нескінченного стану завантаження.

Die Tauri-Hülle für Desktop und Mobile stellt `evaluate_my_lisp` bereit, während der Web/PWA-Build `crates/my-lisp-wasm` nutzt, um die kanonische Rust-Engine über WebAssembly direkt im Browser auszuführen. Beide Host-Grenzen verwenden Single-Pass-Parsing (`eval_parsed_expressions`), sodass der Editorcode nur einmal für AST-Anzeige und Auswertung geparst wird. Wenn WebAssembly in einer Webumgebung nicht geladen werden kann, wird die Anwendung elegant herabgestuft und zeigt einen klaren UI-Fehler anstelle eines endlosen Ladezustands.

The Rust evaluator executes the final expression of a closure and the selected `cond` branch through an explicit trampoline. Tail-recursive programs therefore use constant Rust call-stack space. Closure bodies share immutable AST nodes through `Rc`; the next performance boundary is structural sharing for persistent list values.

Rust evaluator виконує останній вираз closure та вибрану гілку `cond` через явний trampoline. Тому хвостово-рекурсивні програми використовують сталий обсяг Rust call stack. Тіла closure спільно використовують незмінні AST-вузли через `Rc`; наступна межа швидкодії — structural sharing для persistent list values.

Der Rust-Evaluator führt den letzten Closure-Ausdruck und den gewählten `cond`-Zweig über ein explizites Trampolin aus. Tail-rekursive Programme benötigen dadurch konstanten Rust-Call-Stack. Closure-Rümpfe teilen unveränderliche AST-Knoten über `Rc`; die nächste Leistungsgrenze ist strukturelles Teilen persistenter Listenwerte.
