# my-lisp benchmarks · Benchmarks my-lisp · my-lisp-Benchmarks

## English

The benchmark suite runs the same `.my` programs through the ClojureScript prototype and canonical Rust core. It measures parser throughput plus arithmetic, list recursion, direct recursion, and lexical closures. Each case receives 50 warm-up executions; the reported value is average microseconds per operation over 1,000 measured executions.

Run `npm run benchmark`. Set `MY_LISP_BENCH_ITERATIONS` to change the measured iteration count. Results depend on CPU, power mode, compiler version, background activity, and thermal state; compare engines from the same run instead of treating one machine's numbers as universal.

These are microbenchmarks, not product-performance promises. They deliberately include parser and fresh-session allocation in every evaluation operation because that matches the current Language Lab path. Later benchmarks may separately measure persistent REPL sessions, loading `lib/core.my`, tail calls, allocation, and Android devices.

## Українська

Набір benchmark запускає однакові програми `.my` через ClojureScript-прототип і канонічне Rust-ядро. Він вимірює parser, арифметику, рекурсію списків, пряму рекурсію та лексичні замикання. Кожен випадок має 50 прогрівальних запусків; результат — середня кількість мікросекунд на операцію за 1 000 виміряних запусків.

Запуск: `npm run benchmark`. Змінна `MY_LISP_BENCH_ITERATIONS` задає іншу кількість виміряних ітерацій. Результати залежать від CPU, режиму живлення, версії компілятора, фонової активності й температури; порівнюйте рушії з одного запуску, а не сприймайте числа одного комп’ютера як універсальні.

Це microbenchmarks, а не обіцянка швидкодії продукту. Вони навмисно включають parser і створення нової session у кожну eval-операцію, бо це відповідає поточному шляху Language Lab. Пізніше можна окремо вимірювати постійні REPL-сесії, завантаження `lib/core.my`, tail calls, allocations та Android-пристрої.

## Deutsch

Die Benchmark-Suite führt dieselben `.my`-Programme im ClojureScript-Prototyp und im kanonischen Rust-Kern aus. Gemessen werden Parser, Arithmetik, Listenrekursion, direkte Rekursion und lexikalische Closures. Jeder Fall erhält 50 Aufwärmausführungen; ausgegeben werden durchschnittliche Mikrosekunden pro Operation aus 1.000 gemessenen Ausführungen.

Start mit `npm run benchmark`. `MY_LISP_BENCH_ITERATIONS` ändert die Anzahl gemessener Iterationen. Ergebnisse hängen von CPU, Energiemodus, Compiler-Version, Hintergrundlast und Temperatur ab; Engines sollen innerhalb desselben Laufs verglichen werden, statt Zahlen eines Rechners als universell anzusehen.

Dies sind Mikrobenchmarks und keine Leistungszusage für das Produkt. Parser und neue Session-Allokation sind absichtlich Teil jeder Auswertungsoperation, da dies dem aktuellen Language-Lab-Pfad entspricht. Später können dauerhafte REPL-Sitzungen, das Laden von `lib/core.my`, Tail Calls, Allokationen und Android-Geräte getrennt gemessen werden.

## Local baseline · Локальний baseline · Lokale Ausgangsmessung

Windows x86_64, 2026-08-05, release-mode Rust, 1,000 iterations. This is one development-machine run and must be reproduced before making optimization decisions. · Це один запуск на машині розробки, який слід повторювати перед рішеннями щодо оптимізації. · Dies ist ein einzelner Lauf auf dem Entwicklungsrechner und muss vor Optimierungsentscheidungen reproduziert werden.

| Case · Випадок · Fall | CLJS µs/op | Rust µs/op | Rust speedup · Прискорення · Beschleunigung |
|---|---:|---:|---:|
| parser | 90.08 | 8.99 | 10.0× |
| arithmetic | 68.87 | 6.13 | 11.2× |
| lists | 175.12 | 55.30 | 3.2× |
| recursion | 186.53 | 113.15 | 1.6× |
| closures | 52.32 | 10.59 | 4.9× |
