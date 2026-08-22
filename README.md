# my-idea

**A lightweight programming IDE · Легка IDE для програмування · Eine leichtgewichtige Programmier-IDE**

[English](#english) · [Українська](#українська) · [Deutsch](#deutsch)

`my-idea` is a new programming IDE forked from [`my-ide`](https://github.com/juv4uk/my-ide). The editor is built around **CodeMirror 6**, the interface is written in **ClojureScript**, and **Tauri v2 + Rust** provide a small cross-platform desktop and mobile shell.

The main goal is comfortable everyday programming. Our own language experiments are a special built-in **Language Lab**, not a limitation of the IDE.

**my-lisp** is the small independent language developed with the IDE: *a small language that grows itself · маленька мова, що вирощує себе · eine kleine Sprache, die sich selbst wachsen lässt*. Rust supplies the minimal safe semantic machinery; higher-level forms and libraries grow inside my-lisp itself.

## Quick try · Швидко спробувати · Schnell ausprobieren

No installation and no account: **[download the standalone Web IDE](https://github.com/juv4uk/my-idea/releases/latest/download/my-idea-web.html)** and open the downloaded `.html` file in your browser. The editor, themes, programming-language switcher, and my-lisp Language Lab are contained in that one portable file and run locally.

Без встановлення та облікового запису: **[завантажте автономну Web IDE](https://github.com/juv4uk/my-idea/releases/latest/download/my-idea-web.html)** і відкрийте отриманий файл `.html` у браузері. Редактор, теми, перемикач мов програмування та my-lisp Language Lab містяться в одному portable-файлі й працюють локально.

Ohne Installation und Benutzerkonto: **[die eigenständige Web-IDE herunterladen](https://github.com/juv4uk/my-idea/releases/latest/download/my-idea-web.html)** und die geladene `.html`-Datei im Browser öffnen. Editor, Themes, Programmiersprachenumschalter und my-lisp Language Lab befinden sich in dieser einen portablen Datei und laufen lokal.

Just the language, no editor: **[download the standalone my-lisp CLI REPL](https://github.com/juv4uk/my-lisp/releases/latest/download/my-lisp-cli-web.html)** — a single portable `.html` file with a terminal-style REPL for the my-lisp core, running entirely in your browser. See [`docs/quote-tutorial.md`](docs/quote-tutorial.md) for a first walkthrough.

Лише мова, без редактора: **[завантажте автономний my-lisp CLI REPL](https://github.com/juv4uk/my-lisp/releases/latest/download/my-lisp-cli-web.html)** — один portable-файл `.html` з термінальним REPL для ядра my-lisp, що працює повністю в браузері. Перший огляд — у [`docs/quote-tutorial.md`](docs/quote-tutorial.md).

Nur die Sprache, ohne Editor: **[den eigenständigen my-lisp-CLI-REPL herunterladen](https://github.com/juv4uk/my-lisp/releases/latest/download/my-lisp-cli-web.html)** — eine einzelne portable `.html`-Datei mit einem terminalartigen REPL für den my-lisp-Kern, die vollständig im Browser läuft. Ein erster Rundgang steht in [`docs/quote-tutorial.md`](docs/quote-tutorial.md).

## Repository history · Історія репозиторію · Repository-Verlauf

`my-idea` preserves the Git history inherited from earlier development and from its `my-ide` origin. Some old commits and release tags therefore refer to earlier project stages. We keep them intentionally for traceability and never rewrite an existing release tag; when a version number is already occupied, the next free patch version is used. The first Rust-language-core release of `my-idea` is **v0.3.2**.

`my-idea` зберігає Git-історію, успадковану від ранніх етапів розробки та проєкту `my-ide`. Тому частина старих комітів і релізних тегів стосується попередніх етапів проєкту. Ми навмисно залишаємо їх для простежуваності й ніколи не переписуємо наявний релізний тег; якщо номер версії вже зайнятий, використовуємо наступну вільну patch-версію. Перший реліз `my-idea` з Rust-ядром мови — **v0.3.2**.

`my-idea` bewahrt die Git-Historie aus früheren Entwicklungsphasen und aus seinem Ursprung `my-ide`. Deshalb beziehen sich einige ältere Commits und Release-Tags auf frühere Projektstände. Zur Nachvollziehbarkeit bleiben sie bewusst erhalten; ein vorhandenes Release-Tag wird niemals überschrieben. Ist eine Versionsnummer bereits belegt, verwenden wir die nächste freie Patch-Version. Das erste `my-idea`-Release mit Rust-Sprachkern ist **v0.3.2**.

Details: [`docs/versioning.md`](docs/versioning.md)

my-lisp source files use the canonical `.my` extension; see the [trilingual source-file contract](docs/source-files.md).

## English

### What already works

- CodeMirror 6 editor with Clojure highlighting, line numbers, history, bracket matching, folding, completion and diagnostics;
- resizable workspace: sidebar width, right-column width and the console/pane split are drag-adjustable and persist across restarts (v0.13.0);
- local source persistence and a responsive desktop/mobile workspace;
- embedded safe Lisp evaluator (powered by Rust and WebAssembly) with console, parsed-form view, and Markdown/Mermaid preview;
- System Observatory tools (desktop builds): ecosystem conformance check, live my-lisp TCP oracle, engine-vs-oracle comparison, swarm-node status and dashboard, repo.my knowledge graph;
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

Requirements: [Bun](https://bun.sh) 1.3.8 (package manager and script runner), Node.js 20+ (a couple of scripts still shell out to it directly), Java 17+ (for Shadow CLJS), and the platform requirements for Tauri. This repository is developed inside a reproducible [GNU Guix](https://guix.gnu.org) environment (`manifest.scm`); building the WASM engine additionally needs rustup + wasm-pack outside Guix — both are documented step by step in [`AGENTS.md`](AGENTS.md).

```bash
git submodule update --init external/my-lisp   # needed for bun run build / release.sh
bun install
bun run dev
```

```bash
bun run test
bun run check
bun run build
bun run tauri dev
```

Architecture notes and the platform roadmap live in [`docs/README.md`](docs/README.md). Contributions and practical ideas are welcome.

## License · Ліцензія · Lizenz

[MIT](LICENSE)
