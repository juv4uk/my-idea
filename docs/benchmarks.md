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

Windows x86_64, 2026-08-05, release-mode Rust, median of three runs with 5,000 measured iterations per case. CLJS varied more because JIT and garbage-collection activity remain part of the measurement; the median avoids presenting either the best or worst outlier as the baseline.

Windows x86_64, 2026-08-05, Rust у release mode, медіана трьох прогонів по 5 000 виміряних ітерацій на випадок. CLJS коливався сильніше через JIT і garbage collection, які залишаються частиною вимірювання; медіана не видає ані найкращий, ані найгірший викид за baseline.

Windows x86_64, 2026-08-05, Rust im Release-Modus, Median aus drei Läufen mit jeweils 5.000 gemessenen Iterationen pro Fall. CLJS schwankte wegen JIT- und Garbage-Collection-Aktivität stärker; der Median verwendet weder den besten noch den schlechtesten Ausreißer als Ausgangswert.

| Case · Випадок · Fall | CLJS µs/op | Rust µs/op | Rust speedup · Прискорення · Beschleunigung |
|---|---:|---:|---:|
| parser | 62.72 | 8.60 | 7.3× |
| arithmetic | 57.39 | 7.12 | 8.1× |
| lists | 198.26 | 52.40 | 3.8× |
| recursion | 194.92 | 105.36 | 1.8× |
| closures | 48.14 | 10.36 | 4.6× |
