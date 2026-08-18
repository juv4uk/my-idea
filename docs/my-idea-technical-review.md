# Технічний огляд `my-idea`

**Автор огляду:** Manus AI  
**Стан джерел:** гілка `main`, переглянута 18 серпня 2026 року  
**Репозиторій:** [juv4uk/my-idea][1]

## Висновок у двох реченнях

`my-idea` уже має сильнішу ідентичність, ніж «IDE для мого Lisp»: це **local-first programming workspace**, де повсякденний редактор, portable web artifact, safe embedded evaluator і desktop observability можуть співіснувати без того, щоб один режим удавав інший. Language Lab тут не окремий demo-віджет, а справжня багатоплатформна межа: на web — capability-free WASM `my-lisp`, на native — той самий canonical Rust engine через Tauri, а live oracle/swarm/ecosystem data доступні лише там, де вони справді можуть існувати [2] [3].

> Найсильніша ідея `my-idea`: не показувати екосистему як набір репозиторіїв, а дати людині спосіб побачити контракт, evidence і semantic agreement між реалізаціями.

## 1. Продуктова модель: IDE спочатку, лабораторія мови — як надбудова

README дуже чітко фіксує порядок пріоритетів: CodeMirror 6, ClojureScript UI, Tauri v2/Rust shell, local persistence і normal file/project editing — це продуктове ядро. Власні language experiments оформлені як built-in Language Lab, але не повинні перетворювати IDE на нішевий редактор тільки для `my-lisp` [2].

| Площина | Що вже існує | Практичний сенс |
|---|---|---|
| Everyday IDE | Tabs, folder workspace, CodeMirror, themes, i18n, preview | Не треба відкривати окремий інструмент для звичайного коду. |
| Portable web | Один standalone HTML без account/install | Низький бар’єр входу до редактора та Language Lab. |
| Native desktop | Tauri workspace, files, dialogs, ecosystem panels | Реальні local capabilities не маскуються під web features. |
| Language Lab | my-lisp pure/literate modes, console, AST, diagnostics | Мова — observable і доступна для experimentation. |
| System Observatory | Evidence matrix, Oracle, Compare, Swarm, graph | IDE стає interface до живої екосистеми, а не лише text editor. |

Це правильна позиція. Твій `my-lisp` може лишатися deeply integrated, але IDE не мусить чекати, поки мова стане «повною», щоб бути корисною.

## 2. Архітектура: простий і читабельний розподіл відповідальностей

```text
CodeMirror 6 editor
       │ source / change events / diagnostics
       ▼
ClojureScript app
  core.cljs ─ commands.cljs ─ workspace.cljs
       │                │
       │                ├── browser directory/download APIs
       │                ├── WASM my-lisp evaluator
       │                └── Tauri IPC (native only)
       ▼
Rust / Tauri shell
  workspace filesystem · my-lisp Session · oracle · swarm · ecosystem graph
```

`core.cljs` відтворює основну three-pane композицію: workspace/tree+tabs, CodeMirror center, console+AST/preview/observability panel right. Водночас він не змішує весь control flow у одному namespace: actions винесені у `commands.cljs`, workspace semantics — у `workspace.cljs`, rendering ecosystem data — у pure `eco_view.cljs`, а WASM lifecycle — у `wasm.cljs` [4] [5]. Це вже непогана внутрішня модульність для ClojureScript app без зайвого framework.

`editor.cljs` тримає один CodeMirror instance на workspace, дає language-mode extensions для Clojure, Rust, Markdown, Mermaid і text, підключає history, folding, matching, completion і linter. Важливо, що diagnostics не є декоративними: вони викликають `wasm/diagnose` на актуальному editor document [6].

## 3. Language Lab: справді багатоплатформний, а не умовний

У web mode ClojureScript lazily завантажує `my-lisp` wasm-bindgen module через plain-JS dynamic-import shim; інтерфейс обчислення повертає `{value, output, ast, engine}` [7]. У native mode Tauri `evaluate_my_lisp` створює fresh default `my_lisp::Session`, обирає `PureLisp` або `Literate` source mode, виконує embedded Rust engine й повертає ті самі поля [3].

| Середовище | Engine | Доступ до host | Чесна поведінка |
|---|---|---|---|
| Browser / portable HTML | WebAssembly `my-lisp` | Немає desktop file/network capabilities | Працює локально; evaluator доступний без install. |
| Tauri desktop | Canonical Rust `my-lisp` Session | Session default/capability-free | Вбудований execution не отримує тихих system permissions. |
| Live Oracle | Running `my-lisp --tcp --protocol=sexpr` | Лише через native Tauri command | Стан і протокол іншого runtime показані як external oracle. |
| Planned Guile | Optional desktop adapter | Ще не connected | README не вдає, що full Scheme runtime уже працює. |

Це особливо добре з точки зору безпеки. `evaluate_my_lisp` не підхоплює відкритий workspace як implicit capability; code evaluation і file access — окремі канали. README прямо формулює той самий принцип: runtime не отримує silent file/network access [2] [3].

Literate Markdown mode — не косметичний. Native bridge передає `SourceMode::Literate`, а UI має markdown preview, тож technical notes можуть одночасно бути readable document та executable my-lisp material [3] [4]. Це дуже природно з’єднується з твоєю research/knowledge workflow.

## 4. Workspace security і mobile honesty

Tauri workspace API має здорову межу: user chooses root, root canonicalize-иться, а кожен read/save existing path ще раз canonicalize-иться та перевіряється через `starts_with(root)`. Path escape відхиляється; звичайний save працює лише для existing workspace file, а Save As іде через system dialog [3].

Browser mode також не робить вигляд, що він native: folder selection, cached permission-aware file handles та download fallback реалізовані окремо. Android не обіцяє filesystem access, якого ще немає: `choose_workspace` і `save_as_dialog` повертають meaningful «planned Storage Access Framework» limitation [3] [5].

| Режим | Files | Save | Обмеження |
|---|---|---|---|
| Desktop | Chosen workspace only | Existing file / system Save As | Path traversal blocked by canonical root check. |
| Browser | User-selected directory handles, cached where permitted | Browser download / Save As mechanism | After reload, user may reselect workspace. |
| Mobile | No generic folder workspace yet | No native Save As yet | Explicit message instead of fake availability. |

## 5. System Observatory: найоригінальніша product feature

Ecosystem panel не обмежується «зеленими status lights». Для кожного evidence record `eco_view.cljs` показує expected/actual, commit, runner, timestamp, optional environment and note; drill-down формує causal chain:

```text
SOURCE → my-lisp oracle → CML compile → fpga-lisp execute
```

Semantic agreement оголошується тільки якщо всі три implementations мають `pass` і той самий actual result [8]. Додатково panel попереджає, якщо embedded my-lisp revision у `my-idea` не збігається із sibling checkout, і прямо каже використовувати Oracle/Compare для live check, а не вважати bundle абсолютною істиною [8].

Це правильна епістемічна UX-поведінка: **версія, evidence і джерело виконання залишаються видимими**. Для твоєї екосистеми це важливіше, ніж додати ще десять загальних IDE кнопок.

## 6. Knowledge Graph: чесний phase 1, який already має цінність

Knowledge Graph design правильно відмовляється заштовхувати `shiva-sutras`/`my-lisp-panini` у стару symmetric evidence matrix. Matrix відповідає на «чи три implementation узгоджуються щодо requirement?», а graph має показувати directed import/provenance relations і, у майбутньому, claim statuses/drift [9].

Phase 1 already does the correct small thing. `repo_graph.rs` читає `repo.my` через actual my-lisp reader, створює nodes для всіх known siblings навіть без declaration, і виводить coarse edge лише з self-declared `imports ∩ exports` overlap [10]. UI чітко називає graph repo-level phase 1 й не удає, що capability edge already equals claim-level proof [8].

| Що працює зараз | Що свідомо відкладено |
|---|---|
| Six sibling repo nodes, including missing self-declarations | Claim nodes with `statement`, `status`, `scope`, evidence and limitations |
| Capability-overlap edges from machine-readable contracts | Claim import edges from Panini → Shiva Sutras |
| Tolerant missing `repo.my` handling | Upstream status-at-import vs current-status drift highlighting |
| Native-only graph panel | My-idea’s own `repo.my` declaration and self-description |

Найкращий next step тут уже визначений у design doc: tolerant parser for actual `claims-export.yaml` Markdown-shaped data and Panini import records. Але **перед phase 2** я б додав `my-idea/repo.my`: без цього observer itself is visible as an undeclared/missing node, хоча його role as observer/IDE вже conceptually ясна.

## 7. Verification: хороше coverage ядра, але треба розширити real native E2E

Документація фіксує 95 automated tests: 53 Rust tests і 42 Node/Playwright tests. Вони охоплюють Rust language core, CLI, literate mapping, WASM conformance, 100k list stack safety, portable web artifact, PWA wiring, ecosystem panel and CLI-web REPL [11].

| Сильне покриття | Поточна межа |
|---|---|
| WASM engine runs implementation-independent fixture cases | `shadow-cljs compile test` поки має 0 ClojureScript assertions. |
| 100k list checks protect stack-safety claims in wasm/browser | Native adapter has only one direct unit test. |
| Playwright protects portable artifact and ecosystem UI regression | `eco-panel.test.mjs` uses mocked data rather than real Tauri backend. |
| Tauri API names are smoke-checked | Full desktop command → actual sibling repo graph E2E is not yet the documented browser test path. |

Це не скасовує цінність current tests — especially WASM and app artifact coverage. Але для System Observatory наступним verification milestone має бути **real Tauri integration test**: temporary sibling repo layout → actual `ecosystem_status` / `knowledge_graph` → asserted structured output. Інакше UI test може довести layout, але не correctness data derivation.

## 8. Пріоритетні наступні кроки

Перший крок — додати `repo.my` для `my-idea`, позначивши role як observer/IDE, exports як visualized ecosystem status/graph, imports як my-lisp contracts/evidence/repo declarations, і explicit authorities/non-authorities. Це дасть graph саморефлексивність без hardcoded exception.

Другий — реалізувати claim-level graph як planned, але з revision-aware detail view: current upstream status, downstream `status_at_import`, revision, timestamp and direct source link. Найцінніший visual signal там не круги й стрілки, а саме **drift alert**, коли downstream reasoning покладається на старий статус claim-а.

Третій — додати one or two real native integration tests for Tauri ecosystem commands and a small CLJS unit layer for pure command/state functions. Не тому, що existing tests слабкі, а тому що новий System Observatory становить product-level contract, який now deserves backend-to-frontend confidence beyond mocked panel data.

Четвертий — не поспішати підключати багато runtimes. `my-lisp` Lab already has clear value. Якщо з’явиться Guile adapter, йому варто давати той самий explicit capability contract і output structure, а не дозволяти arbitrary shell or workspace access because it is «desktop only».

## Підсумок

`my-idea` already має дві переконливі ролі, які не конфліктують: **легкий editor for everyday programming** і **проникний інтерфейс до екосистеми формальної мови/knowledge/implementation evidence**. Найцікавіше в ньому не CodeMirror і не Tauri окремо, а те, що UI дозволяє людині побачити розриви між embedded engine, live oracle, compiler path і FPGA execution without hiding uncertainty.

Якщо `my-lisp` — це semantic substrate, CML — compiler bridge, а fpga-lisp — independent hardware manifestation, то `my-idea` поступово стає **людською поверхнею, де ці шари можна спостерігати, порівнювати й не плутати між собою**.

## References

[1]: https://github.com/juv4uk/my-idea "my-idea repository"
[2]: https://github.com/juv4uk/my-idea/blob/main/README.md "my-idea README"
[3]: https://github.com/juv4uk/my-idea/blob/main/src-tauri/src/lib.rs "Tauri command surface and workspace boundary"
[4]: https://github.com/juv4uk/my-idea/blob/main/src-cljs/my_idea/core.cljs "ClojureScript application shell"
[5]: https://github.com/juv4uk/my-idea/blob/main/src-cljs/my_idea/commands.cljs "Command/controller layer"
[6]: https://github.com/juv4uk/my-idea/blob/main/src-cljs/my_idea/editor.cljs "CodeMirror integration"
[7]: https://github.com/juv4uk/my-idea/blob/main/src-cljs/my_idea/wasm.cljs "WASM my-lisp bridge"
[8]: https://github.com/juv4uk/my-idea/blob/main/src-cljs/my_idea/eco_view.cljs "Observatory and graph renderers"
[9]: https://github.com/juv4uk/my-idea/blob/main/docs/knowledge-graph-design.md "Knowledge Graph design"
[10]: https://github.com/juv4uk/my-idea/blob/main/src-tauri/src/ecosystem/repo_graph.rs "Phase-1 repo graph backend"
[11]: https://github.com/juv4uk/my-idea/blob/main/docs/testing.md "Testing matrix"
