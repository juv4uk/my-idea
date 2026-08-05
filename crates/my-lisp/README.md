# my-lisp core · Ядро my-lisp · my-lisp-Kern

> **A small language that grows itself. · Маленька мова, що вирощує себе. · Eine kleine Sprache, die sich selbst wachsen lässt.**

An independent, capability-free Rust library for **my-lisp**, the small Lisp developed alongside the my-idea IDE. The first milestone implements McCarthy's seven primitives and keeps parser, runtime values, environments, diagnostics, and tests outside the IDE shell.

Незалежна Rust-бібліотека без доступу до системних можливостей для **my-lisp** — маленького Lisp, який розвивається поруч з IDE my-idea. Перший етап реалізує сім примітивів Маккарті та тримає парсер, значення виконання, середовища, діагностику й тести поза оболонкою IDE.

Eine unabhängige Rust-Bibliothek ohne Systemzugriffe für **my-lisp**, den kleinen Lisp, der zusammen mit der my-idea-IDE entwickelt wird. Der erste Meilenstein implementiert McCarthys sieben Primitive und hält Parser, Laufzeitwerte, Umgebungen, Diagnosen und Tests außerhalb der IDE-Hülle.

## Contract · Контракт · Vertrag

- `quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond`
- UTF-8 symbols, comments, strings, numbers, lists, and apostrophe quote syntax
- UTF-8-символи, коментарі, рядки, числа, списки та скорочення цитування через апостроф
- UTF-8-Symbole, Kommentare, Zeichenketten, Zahlen, Listen und Apostroph-Kurzsyntax für Zitate
- structured errors with source spans · структуровані помилки з діапазонами коду · strukturierte Fehler mit Quellbereichen
- no Tauri, filesystem, network, or UI dependency · без залежності від Tauri, файлів, мережі чи UI · keine Abhängigkeit von Tauri, Dateisystem, Netzwerk oder UI

Rust contains the minimal semantic machinery, including lexical `lambda`. Derived forms and library functions will be bootstrapped in the language itself.

Rust містить мінімальну семантичну механіку, включно з лексичною `lambda`. Похідні форми та бібліотечні функції розгортатимуться самою мовою.

Rust enthält die minimale semantische Mechanik einschließlich lexikalischem `lambda`. Abgeleitete Formen und Bibliotheksfunktionen werden in der Sprache selbst gebootstrappt.

## Test · Тестування · Test

```powershell
cargo test --manifest-path crates/my-lisp/Cargo.toml
```

The crate starts at its own library version `0.1.0`; the containing IDE release is versioned independently.

Крейт починає власну версію бібліотеки з `0.1.0`; реліз IDE, що його містить, версіонується незалежно.

Das Crate beginnt mit der eigenen Bibliotheksversion `0.1.0`; das enthaltene IDE-Release wird unabhängig versioniert.
