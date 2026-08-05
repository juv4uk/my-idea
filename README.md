# my-idea

**A lightweight programming IDE · Легка IDE для програмування · Eine leichtgewichtige Programmier-IDE**

[English](#english) · [Українська](#українська) · [Deutsch](#deutsch)

`my-idea` is a new programming IDE forked from [`my-ide`](https://github.com/juv4uk/my-ide). The editor is built around **CodeMirror 6**, the interface is written in **ClojureScript**, and **Tauri v2 + Rust** provide a small cross-platform desktop and mobile shell.

The main goal is comfortable everyday programming. Our own language experiments are a special built-in **Language Lab**, not a limitation of the IDE.

## English

### What already works

- CodeMirror 6 editor with Clojure highlighting, line numbers, history, bracket matching, folding, completion and diagnostics;
- local source persistence and a responsive desktop/mobile workspace;
- embedded safe Lisp evaluator with console and parsed-form view;
- English, Ukrainian and German interface;
- web build plus the Tauri foundation for Windows, Linux, macOS and mobile.

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

Architecture notes live in [`docs/README.md`](docs/README.md). Contributions and practical ideas are welcome.

## License · Ліцензія · Lizenz

[MIT](LICENSE)
