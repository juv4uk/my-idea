# MYIDEA-KNOWLEDGE-GRAPH — дизайн

Статус: **DRAFT**, перший прохід. Реалізує `tasks.my`'s `MYIDEA-KNOWLEDGE-GRAPH`
("Build Knowledge Graph tab: visualize cross-repo epistemic chains").

## Навіщо це і чому не evidence-matrix

Існуюча `eco-view.cljs` вже візуалізує один зріз екосистеми — таблицю
`requirement × implementation` для трьох репо (`my-lisp`/`cml`/`fpga-lisp`),
жорстко зашиту в `mod.rs`/`ecosystem-html`. Це добре працює для "чи
семантика одна й та сама в трьох двигунах" — вузький, симетричний випадок.

`shiva-sutras`/`my-lisp-panini` — інша форма даних: не симетрична матриця,
а **орієнтований граф залежностей** між репозиторіями (хто в кого
"імпортує" claim) і **окремі епістемічні твердження** зі статусом
(`PROVED`/`SUPPORTED`/`FALSIFIED`/`UNRESOLVED`/...), а не pass/fail.
Втискати це в існуючу 3-колонкову таблицю — неправильна форма даних.
Потрібен окремий, справді графовий rendering.

## Джерела даних (вже існують, нічого не винаходимо)

1. **`repo.my`** (Swarm Contract v0.1, часткове прийняття — уже є в
   `fpga-lisp`, `shiva-sutras`, `my-lisp-panini`; відсутнє в `my-lisp`,
   `cml`, `my-idea` — сам `my-idea` теж має відкриту
   `MYIDEA-SWARM-CONTRACT-01`). S-expr формат, той самий reader, яким
   `contracts.rs`/`evidence.rs` вже парсять `.my`-файли:
   ```
   (repository
     (id shiva-sutras)
     (role research-lab)
     (exports (shiva-claims))
     (imports (panini-claims))
     (capabilities (sanskrit panini slp1 gretil provenance epistemic-pipeline))
     (authorities (...))
     (non-authorities (...)))
   ```
   Це дає **вузли графа** (репо, з роллю й можливостями) і **грубі ребра**
   (repo A imports capability X, repo B exports capability X).

2. **`docs/claims-export.yaml`** (`shiva-sutras`) — реєстр окремих claims
   з `status`/`scope`/`evidence`/`limitations`. **Важливо**: файл
   називається `.yaml`, але його реальний вміст — Markdown-заголовки
   (`### SS-CANON-001 — ...`) з жирними полями-буллетами, **не** валідний
   YAML-список записів, як описано в `docs/epistemic-coordination.md`'s
   приклад (`claim-id: ...` формат). Це розбіжність між задокументованим
   і фактичним форматом — не наша справа виправляти чужий репозиторій
   (`shiva-sutras` — upstream-авторитет, ми не даємо йому архітектурних
   порад, per `epistemic-coordination.md` §3), тож парсер тут читає
   **фактичний** Markdown-формат, толерантно, і не падає, якщо порядок
   полів чи набір ключів трохи зміниться.

3. **`ecosystem/imports/*.my`** (`my-lisp-panini`) — конкретні
   спожиті claim ID з `status_at_import`/`revision`, тобто **точні
   ребра claim-рівня** (не просто "репо A залежить від репо B", а
   "my-lisp-panini спожив SS-PRATYAHARA-001 на статусі SUPPORTED,
   revision X").

## Модель даних

Два рівні зуму, обидва в одному графі:

```
RepoNode {
  id, role, capabilities: [String],
  authorities: [String], non_authorities: [String],
  found: bool  // чи є на диску як сусід my-idea
}

RepoEdge {
  from: repo_id, to: repo_id,
  via_capability: String  // яка capability з'єднує (exports ∩ imports)
}

ClaimNode {
  id,               // "SS-MARKERS-001"
  owner_repo: repo_id,
  statement: String,
  scope: String,
  status: String,   // PROVED-IN-MODEL / SUPPORTED / FALSIFIED / UNRESOLVED / RESOLVED / ...
  evidence: [String],
  limitations: Option<String>,
}

ClaimEdge {
  claim_id, consumer_repo: repo_id,
  status_at_import: String,  // може відставати від claim's поточного статусу — це і є "дрейф", який граф має показувати
  revision: String,
}
```

`ClaimEdge.status_at_import != ClaimNode.status` — це саме той сигнал
"upstream impact" з §7 `epistemic-coordination.md`: claim змінився,
downstream ще не наздогнав. Граф має **явно підсвічувати** цю
розбіжність — вона важливіша за будь-яку іншу деталь на панелі.

## Бекенд (Rust)

Новий модуль `src-tauri/src/ecosystem/repo_graph.rs`, той самий стиль,
що й `evidence.rs`/`contracts.rs` (перевикористовує `parse_alist`/
`assoc`/`as_list`/`symbol_name` з `contracts.rs`, не новий парсер):

```rust
pub fn scan_repo_declarations(siblings_root: &Path) -> Vec<RepoNode>;
pub fn derive_capability_edges(nodes: &[RepoNode]) -> Vec<RepoEdge>;
pub fn scan_claims(shiva_sutras_path: &Path) -> Vec<ClaimNode>;      // claims-export.yaml, tolerant parse
pub fn scan_claim_imports(panini_path: &Path) -> Vec<ClaimEdge>;     // ecosystem/imports/*.my
```

`siblings_root()` (вже є в `mod.rs`) розширюється списком репо — зараз
жорстко `my-lisp`/`fpga-lisp`/`cml`; додати `shiva-sutras`,
`my-lisp-panini` (обидва вже є сусідами на диску). Новий Tauri-командний
шар — окрема команда `knowledge_graph_status`, **не** розширення
`ecosystem_status` — різні панелі, різний ритм оновлення (claims
змінюються рідше за evidence-run), не варто змушувати одне тягнути інше.

## Фронтенд (ClojureScript)

Новий `eco-view/knowledge-graph-html` (та сама pure-function конвенція,
без доступу до `@state`). Рендеринг — inline SVG (той самий підхід, що
`causal-chain-html` для стрілок, лише масштабований до N вузлів):

- Вузли-репо розташовані по колонках за `role` (research-lab →
  knowledge-compiler → observer/IDE тощо) — не force-directed layout
  (без нової JS-залежності), простий детермінований layout: одна колонка
  на роль, вузли всередині колонки — за алфавітом.
- Ребра репо — тонкі лінії, підписані capability.
  Ребра claim-рівня — товщі, кольорові за статусом claim'а
  (зелений `PROVED`/`RESOLVED`/`SUPPORTED`, жовтий `UNRESOLVED`, червоний
  `FALSIFIED`), і **обведені пунктиром**, якщо
  `status_at_import != поточний status` (дрейф).
- Клік по вузлу claim — розкриває `statement`/`scope`/`evidence`/
  `limitations` тим самим `eco-fixture`-подібним detail-панелі патерном,
  що вже є для `fixture-detail-html`.

Кнопка `🕸 Knowledge Graph` поруч із наявними `🔭 Ecosystem`/`🔮 Oracle`/
`⚖ Compare`/`🐝 Swarm` у `core.cljs`, той самий `(workspace/native?)`
guard (десктопна фіча, не веб-збірка).

## Фазування

1. **Фаза 1 (цей коміт)**: repo-рівень тільки — `repo_graph.rs` читає
   наявні `repo.my` (3 з 6 репо мають), рендерить вузли + capability-ребра.
   Відсутні `repo.my` (my-lisp/cml/my-idea) показуються як "not declared"
   вузли, не як помилка — той самий толерантний стиль, що `evidence.rs`
   для відсутньої `evidence/`.
2. **Фаза 2**: claim-рівень — парсер `claims-export.yaml` (толерантний
   до Markdown-формату) + `ecosystem/imports/*.my` для реальних ребер
   і дрейф-підсвітки.
3. **Фаза 3**: коли `my-idea` сам собі напише `repo.my`
   (`MYIDEA-SWARM-CONTRACT-01`), граф стає повністю self-describing —
   my-idea бачить власне місце в ньому, не хардкодить себе окремо.

## Що це NOT

- Не архітектурна порада `shiva-sutras`/`my-lisp-panini` — граф лише
  **показує** те, що ці репозиторії вже самі задекларували й
  експортували. `my-idea` — спостерігач, не учасник епістемічного
  дослідження (див. `AGENTS.md`'s "Subagents and specialist models").
- Не заміна `evidence-matrix`/`ecosystem-html` — окрема панель, окрема
  форма даних.
