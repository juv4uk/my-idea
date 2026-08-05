# my-idea documentation · Документація · Dokumentation

- [Versioning and inherited history · Версіонування та успадкована історія · Versionierung und übernommene Historie](versioning.md)
- [my-lisp source files · Файли my-lisp · my-lisp-Quelldateien](source-files.md)
- [Language core · Ядро мови · Sprachkern](language-core.md)
- [Android releases · Android-релізи · Android-Releases](android-release.md)
- [Platform roadmap · Дорожня карта платформ · Plattform-Roadmap](platform-roadmap.md)

## Product boundary · Межі продукту · Produktgrenze

`my-idea` is a general programming IDE. Editing files and projects is the product core. Language development is an advanced built-in tool called **Language Lab**.

`my-idea` — універсальна IDE для програмування. Ядро продукту — робота з файлами та проєктами. Розробка мов є розширеним вбудованим інструментом **Language Lab**.

`my-idea` ist eine allgemeine Programmier-IDE. Dateien und Projekte bilden den Kern. Sprachentwicklung ist das erweiterte integrierte Werkzeug **Language Lab**.

## Architecture · Архітектура · Architektur

```mermaid
flowchart LR
  UI["ClojureScript UI"] --> CM["CodeMirror 6 editor"]
  CM --> FILES["Files and projects"]
  CM --> LAB["Language Lab"]
  LAB --> SAFE["Embedded safe Lisp evaluator"]
  LAB -. "optional desktop adapter" .-> GUILE["GNU Guile"]
  UI --> TAURI["Tauri v2 / Rust shell"]
```

- `src-cljs/my_idea/editor.cljs` owns the reusable CodeMirror 6 integration.
- `src-cljs/my_idea/core.cljs` renders the current workspace.
- `src-cljs/my_idea/language.cljs` is a deliberately small capability-free Lisp evaluator.
- `src-tauri/` is the native boundary. Future external runtimes belong behind explicit commands here.

## Runtime policy · Політика виконання · Laufzeitrichtlinie

The embedded evaluator uses only known commands and has no filesystem or network primitives. Guile support is planned as optional, detected at runtime, and restricted to an explicit workspace. Web and mobile builds keep the embedded backend.

Вбудований інтерпретатор знає лише дозволені команди й не має примітивів файлової системи або мережі. Guile буде необов’язковим, визначатиметься під час запуску та працюватиме лише з явно вибраною робочою папкою. Web і mobile використовують вбудований бекенд.

Der eingebettete Interpreter kennt nur freigegebene Befehle und besitzt keine Datei- oder Netzwerkprimitive. Guile bleibt optional, wird zur Laufzeit erkannt und auf einen ausdrücklich gewählten Arbeitsbereich begrenzt. Web und Mobile verwenden das eingebettete Backend.
