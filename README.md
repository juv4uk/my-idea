# my-idea

**A lightweight programming IDE · Легка IDE для програмування · Eine leichtgewichtige Programmier-IDE**

[English](#english) · [Українська](#українська) · [Deutsch](#deutsch)

`my-idea` is a new programming IDE forked from [`my-ide`](https://github.com/juv4uk/my-ide). The editor is built around **CodeMirror 6**, the interface is written in **ClojureScript**, and **Tauri v2 + Rust** provide a small cross-platform desktop and mobile shell.

The main goal is comfortable everyday programming. Our own language experiments are a special built-in **Language Lab**, not a limitation of the IDE.

**my-lisp** is the small independent language developed with the IDE: *a small language that grows itself · маленька мова, що вирощує себе · eine kleine Sprache, die sich selbst wachsen lässt*. Rust supplies the minimal safe semantic machinery; higher-level forms and libraries grow inside my-lisp itself.

## Repository history · Історія репозиторію · Repository-Verlauf

`my-idea` preserves the Git history inherited from earlier development and from its `my-ide` origin. Some old commits and release tags therefore refer to earlier project stages. We keep them intentionally for traceability and never rewrite an existing release tag; when a version number is already occupied, the next free patch version is used. The first Rust-language-core release of `my-idea` is **v0.3.2**.

`my-idea` зберігає Git-історію, успадковану від ранніх етапів розробки та проєкту `my-ide`. Тому частина старих комітів і релізних тегів стосується попередніх етапів проєкту. Ми навмисно залишаємо їх для простежуваності й ніколи не переписуємо наявний релізний тег; якщо номер версії вже зайнятий, використовуємо наступну вільну patch-версію. Перший реліз `my-idea` з Rust-ядром мови — **v0.3.2**.

`my-idea` bewahrt die Git-Historie aus früheren Entwicklungsphasen und aus seinem Ursprung `my-ide`. Deshalb beziehen sich einige ältere Commits und Release-Tags auf frühere Projektstände. Zur Nachvollziehbarkeit bleiben sie bewusst erhalten; ein vorhandenes Release-Tag wird niemals überschrieben. Ist eine Versionsnummer bereits belegt, verwenden wir die nächste freie Patch-Version. Das erste `my-idea`-Release mit Rust-Sprachkern ist **v0.3.2**.

Details: [`docs/versioning.md`](docs/versioning.md)

## English

### What already works

- CodeMirror 6 editor with Clojure highlighting, line numbers, history, bracket matching, folding, completion and diagnostics;
- local source persistence and a responsive desktop/mobile workspace;
- embedded safe Lisp evaluator with console and parsed-form view;
- English, Ukrainian and German interface;
- installable offline PWA plus the Tauri foundation for Windows, Linux, macOS and mobile.

### Direction

Normal file/project editing comes first. Language Lab will grow alongside it: the embedded evaluator works everywhere, while optional desktop runtimes such as **GNU Guile** can later provide a full Scheme REPL through a narrow Tauri adapter. A runtime never receives silent file or network access.

## Українська

`my-idea` — легка IDE для звичайного програмування. CodeMirror 6 відповідає за редактор, ClojureScript — за інтерфейс, а Tauri v2 і Rust — за кросплатформну оболонку.

Розробка власних Lisp-подібних мов — наша особлива вбудована лабораторія, але не обмеження програми. Спочатку розвиваємо файли, проєкти, пошук, команди й комфортний редактор. Вбудований безпечний інтерпретатор працюватиме всюди; Guile планується як необов’язковий локальний Scheme-бекенд для настільних систем.

## Deutsch

`my-idea` ist eine leichtgewichtige IDE für die alltägliche Programmierung. CodeMirror 6 bildet den Editor, ClojureScript die Oberfläche und Tauri v2 mit Rust die plattformübergreifende Hülle.

Die Entwicklung eigener Lisp-artiger Sprachen ist unser besonderes eingebautes Sprachlabor, schränkt die IDE aber nicht ein. Dateien, Projekte, Suche, Befehle und ein komfortabler Editor stehen zuerst. Der sichere eingebettete Interpreter läuft überall; Guile ist als optionales lokales Scheme-Backend für Desktop-Systeme vorgesehen.

## Development · Розробка · Entwicklung

Requirements: Node.js 20+, Java 17+ (for Shadow CLJS), and the platform requirements for Tauri.

```bash
npm install
npm run dev
```

```bash
npm test
npm run check
npm run build
npm run tauri dev
```

Architecture notes and the platform roadmap live in [`docs/README.md`](docs/README.md). Contributions and practical ideas are welcome.

## License · Ліцензія · Lizenz

[MIT](LICENSE)
