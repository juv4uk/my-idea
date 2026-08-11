(ns my-idea.core
  (:require [cljs.pprint :as pprint] [clojure.string :as str]
            [my-idea.editor :as editor]
            [my-idea.preview :as preview]
            [my-idea.wasm :as wasm] [my-idea.workspace :as workspace]))

(def demo-source
  "; my-lisp · Rust/CLJS shared contract · спільний контракт · gemeinsamer Vertrag\n(def greeting \"Hello · Привіт · Hallo\")\n(def second (lambda (values) (car (cdr values))))\n(cons greeting (cons (second '(radio antenna)) '()))")

(def markdown-demo
  "# my-lisp literate document\n\nThis is a standard markdown document that mixes prose and code.\n\n```my-lisp\n;; This code block is extracted and evaluated by the engine!\n(def text \"Hello from Literate my-lisp!\")\ntext\n```\n")

(def mermaid-demo
  "graph TD\n    A[Welcome] -->|Evaluate| B(my-lisp)\n    B --> C{Platform}\n    C -->|Desktop| D[Tauri]\n    C -->|Web| E[WASM]\n")

(defonce state (atom {:language (or (.getItem js/localStorage "my-idea:language") "uk")
                      :theme (or (.getItem js/localStorage "my-idea:theme") "auto")
                      :root nil
                      :tree []
                      :open-paths []
                      :active-path nil
                      :documents {}
                      :output ["Ready · Готово · Bereit"] :ast "[]" :error? false :sidebar? true
                      :observatory nil}))
(def messages
  {"en" {:open "Open folder" :new-file "New File" :save "Save" :save-as "Save As" :run "Run" :files "Explorer" :console "Console" :ast "Language Lab / AST" :preview "Preview"
         :themes {"auto" "Auto" "light" "Day" "dark" "Night" "sepia" "Sepia" "signal" "Signal" "amber" "Amber" "forest" "Forest"}}
   "uk" {:open "Відкрити папку" :new-file "Новий файл" :save "Зберегти" :save-as "Зберегти як" :run "Запустити" :files "Файли" :console "Консоль" :ast "Лабораторія мов / AST" :preview "Попередній перегляд"
         :themes {"auto" "Авто" "light" "День" "dark" "Ніч" "sepia" "Сепія" "signal" "Сигнал" "amber" "Бурштин" "forest" "Ліс"}}
   "de" {:open "Ordner öffnen" :new-file "Neue Datei" :save "Speichern" :save-as "Speichern unter" :run "Starten" :files "Explorer" :console "Konsole" :ast "Sprachlabor / AST" :preview "Vorschau"
         :themes {"auto" "Auto" "light" "Tag" "dark" "Nacht" "sepia" "Sepia" "signal" "Signal" "amber" "Bernstein" "forest" "Wald"}}})
(defn- t [key] (get-in messages [(:language @state) key]))
(defn- esc [x] (-> (str x) (str/replace "&" "&amp;") (str/replace "<" "&lt;") (str/replace ">" "&gt;") (str/replace "\"" "&quot;")))
(defn- active-doc [] (get-in @state [:documents (:active-path @state)]))
(def languages ["uk" "de" "en"])
(def themes ["auto" "light" "dark" "sepia" "signal" "amber" "forest"])
(def programming-languages ["my-lisp" "clojurescript" "rust" "markdown" "mermaid" "text"])
(def programming-language-labels {"my-lisp" "my-lisp" "clojurescript" "ClojureScript" "rust" "Rust" "markdown" "Markdown" "mermaid" "Mermaid" "text" "Text"})
(def language-labels {"uk" "UA" "de" "DE" "en" "EN"})
(def theme-icons {"auto" "◐" "light" "☀" "dark" "☾" "sepia" "◉" "signal" "⌁" "amber" "◆" "forest" "♣"})
(defn- next-value [values current]
  (let [index (or (first (keep-indexed #(when (= %2 current) %1) values)) -1)]
    (get values (mod (inc index) (count values)))))
(defn- apply-theme! [theme]
  (set! (.. js/document -documentElement -dataset -theme) theme)
  (.setItem js/localStorage "my-idea:theme" theme))
(declare render! open-file!)

(defn- persist! [] (workspace/remember! @state))
(defn- refresh-tree! []
  (-> (workspace/invoke! "list_workspace" {})
      (.then #(do (swap! state assoc :tree (js->clj % :keywordize-keys true)) (persist!) (render!)))
      (.catch #(do (swap! state assoc :output [(str %)] :error? true) (render!)))))
(defn- choose-browser-workspace! []
  (let [input (.createElement js/document "input")]
    (set! (.-type input) "file")
    (set! (.-multiple input) true)
    (.setAttribute input "webkitdirectory" "")
    (.addEventListener input "change"
      (fn [event]
        (let [files (vec (array-seq (.. event -target -files)))]
          (when (seq files)
            (-> (js/Promise.all (clj->js (map #(.text %) files)))
                (.then (fn [contents]
                         (swap! state workspace/open-browser-workspace files (js->clj contents))
                         (persist!)
                         (render!)))
                (.catch #(do (swap! state assoc :output [(str %)] :error? true) (render!))))))))
    (.click input)))
(defn- open-file! [path]
  (if (get-in @state [:documents path])
    (do (swap! state assoc :active-path path) (persist!) (render!))
    (if (workspace/native?)
      (-> (workspace/invoke! "read_workspace_file" {:path path})
          (.then #(do (swap! state workspace/open-document path %)
                      (persist!)
                      (render!)))
          (.catch #(do (swap! state assoc :output [(str %)] :error? true) (render!))))
      (if-let [p (workspace/read-file-from-handle path)]
        (-> p
            (.then (fn [contents]
                     (swap! state workspace/open-document path contents)
                     (persist!)
                     (render!)))
            (.catch #(do (swap! state assoc :output [(str %)] :error? true) (render!))))
        ;; Handle is gone (page was reloaded) — prompt to re-open folder
        (do (swap! state assoc
                   :output [(case (:language @state)
                              "uk" "📁 Після перезавантаження сторінки потрібно знову відкрити папку (кнопка 📁 зліва)"
                              "de" "📁 Nach dem Neuladen muss der Ordner erneut geöffnet werden (Schaltfläche 📁 links)"
                              "📁 After page reload, re-open the folder using the 📁 button on the left")]
                   :error? false)
            (render!))))))

(defn- new-file! []
  (let [name (js/prompt (case (:language @state)
                          "uk" "Назва файлу:"
                          "de" "Dateiname:"
                          "File name:") "untitled.my")]
    (when (and name (not (str/blank? name)))
      (let [path (if (:root @state) name (str/trim name))]
        (swap! state workspace/open-document path "")
        (persist!)
        (render!)))))
(defn- choose-workspace! []
  (if (workspace/native?)
    (-> (workspace/invoke! "choose_workspace" {})
        (.then #(when % (swap! state assoc :root % :tree [] :open-paths [] :active-path nil :documents {}) (refresh-tree!))))
    (choose-browser-workspace!)))

(defn- save! []
  (when-let [path (:active-path @state)]
    (let [contents (editor/source)]
      (if (and (workspace/native?) (:root @state))
        (-> (workspace/invoke! "save_workspace_file" {:path path :contents contents})
            (.then #(do (swap! state update-in [:documents path] merge {:contents contents :saved contents :dirty? false}) (render!)))
            (.catch #(do (swap! state assoc :output [(str %)] :error? true) (render!))))
        (do (workspace/download! path contents)
            (swap! state update-in [:documents path] merge {:contents contents :saved contents :dirty? false})
            (render!))))))

(defn- save-as! []
  (let [contents (editor/source)
        path (or (:active-path @state) "untitled.my")]
    (if (workspace/native?)
      (-> (workspace/invoke! "save_as_dialog" {:path path :contents contents})
          (.then (fn [new-path]
                   (when new-path
                     (swap! state assoc :active-path new-path)
                     (swap! state update-in [:documents new-path] merge {:contents contents :saved contents :dirty? false})
                     (render!))))
          (.catch #(do (swap! state assoc :output [(str %)] :error? true) (render!))))
      (workspace/save-as-browser! path contents
        (fn [new-path]
          (when new-path
            (swap! state assoc :active-path new-path)
            (swap! state update-in [:documents new-path] merge {:contents contents :saved contents :dirty? false})
            (render!)))))))

(defn- handle-eval-result! [result]
  (let [{:keys [value output ast engine]} (js->clj result :keywordize-keys true)]
    (swap! state assoc :output (into [(str engine)] (conj (vec output) (str "=> " value)))
           :ast ast :error? false)
    (render!)))

(defn- handle-eval-error! [error]
  (swap! state assoc :output [(str error)] :ast "Parse/evaluation stopped" :error? true)
  (render!))

(defn- check-ecosystem! []
  (-> (workspace/invoke! "ecosystem_status" {})
      (.then #(let [status (js->clj % :keywordize-keys true)]
                (swap! state assoc :observatory status :error? false)
                (render!)))
      (.catch #(do (swap! state assoc :output [(str %)] :error? true) (render!)))))

(defn- version-label [version]
  (if version (str (:major version) "." (:minor version)) "—"))

(defn- repo-card [repo title detail-label detail]
  (str "<article class='repo-card'><div class='repo-card-head'><span class='repo-dot "
       (if (:found repo) "found" "missing") "'></span><h2>" title "</h2></div>"
       "<dl><dt>Status</dt><dd>" (if (:found repo) "Local repository" "Not found") "</dd>"
       "<dt>Branch</dt><dd><code>" (esc (or (:branch repo) "—")) "</code></dd>"
       "<dt>Commit</dt><dd><code>" (esc (or (:sha repo) "—")) "</code></dd>"
       "<dt>" detail-label "</dt><dd>" (esc detail) "</dd></dl>"
       "<small class='repo-path'>" (esc (:path repo)) "</small></article>"))

(defn- observatory-html [status]
  (let [{:keys [my-lisp my-lisp-contract cml cml-compatibility cml-status
                fpga-lisp fpga-lisp-contract compatibility]} status
        language-ok (:language-match compatibility)
        isa-ok (:isa-match compatibility)
        compatible? (and language-ok isa-ok)
        compiler (or (:compiler-version cml-compatibility) [])]
    (str "<section class='observatory'><div class='observatory-title'><div><span class='eyebrow'>SYSTEM OBSERVATORY</span>"
         "<h1>Ecosystem snapshot</h1><p>Local, offline-first view of contracts and revisions.</p></div>"
         "<div class='observatory-actions'><button id='close-observatory'>← Editor</button><button id='refresh-ecosystem' class='run'>↻ Run ecosystem check</button></div></div>"
         "<div class='repo-grid'>"
         (repo-card my-lisp "my-lisp" "Language contract" (version-label (:version my-lisp-contract)))
         (repo-card cml "cml" "Compiler" (if (seq compiler) (str/join "." compiler) "—"))
         (repo-card fpga-lisp "fpga-lisp" "ISA contract" (version-label (:version fpga-lisp-contract)))
         "</div><section class='compatibility-card " (if compatible? "compatible" "mismatch") "'>"
         "<div><span class='compatibility-icon'>" (if compatible? "✓" "!") "</span><div><span class='eyebrow'>COMPATIBILITY</span><h2>"
         (if compatible? "Contracts agree" "Contract mismatch") "</h2></div></div>"
         "<div class='compatibility-checks'><span>" (if language-ok "✓" "×") " Language "
         (version-label (:language-contract cml-compatibility)) " → " (version-label (:version my-lisp-contract)) "</span>"
         "<span>" (if isa-ok "✓" "×") " ISA " (version-label (:isa-contract cml-compatibility)) " → "
         (version-label (:version fpga-lisp-contract)) "</span>"
         (when cml-status (str "<span>Tier-1 skips: " (:tier1-skips-remaining cml-status) " · CI: " (esc (:ci-status cml-status)) "</span>"))
         "</div></section></section>")))

(defn- execute! []
  (let [source (editor/source)
        mode (or (:language-mode (active-doc)) "text")]
    (swap! state workspace/update-active source)
    (if (contains? #{"my-lisp" "markdown"} mode)
      (if (workspace/native?)
        ;; Desktop (Tauri) — canonical Rust engine via IPC
        ;; Десктоп (Tauri) — канонічний Rust-рушій через IPC
        ;; Desktop (Tauri) – kanonische Rust-Engine über IPC
        (-> (workspace/invoke! "evaluate_my_lisp" {:source source :mode mode})
            (.then handle-eval-result!)
            (.catch handle-eval-error!))
        (cond
          (wasm/ready?)
          ;; Web — same Rust engine compiled to WASM
          ;; Веб — той самий Rust-рушій, зкомпільований до WASM
          ;; Web – dieselbe Rust-Engine als WASM kompiliert
          (-> (wasm/evaluate source mode)
              (.then handle-eval-result!)
              (.catch handle-eval-error!))
              
          (wasm/failed?)
          ;; WebAssembly failed to load (e.g. no support or CSP blocked)
          ;; WebAssembly не вдалося завантажити (не підтримується або заблоковано CSP)
          ;; WebAssembly konnte nicht geladen werden (nicht unterstützt oder durch CSP blockiert)
          (swap! state assoc
                 :output [(case (:language @state)
                            "uk" "Ваш браузер не підтримує WebAssembly (або заблоковано) · code execution unavailable"
                            "de" "Ihr Browser unterstützt WebAssembly nicht (oder blockiert) · code execution unavailable"
                            "Your browser does not support WebAssembly (or blocked) · code execution unavailable")]
                 :error? true)
                 
          :else
          ;; Fallback while WASM is still loading
          ;; Fallback поки WASM завантажується
          ;; Fallback während das WASM noch lädt
          (swap! state assoc
                 :output ["my-lisp WASM engine is loading… · очікуйте завершення завантаження · WASM wird geladen…"]
                 :error? false)))
      (swap! state assoc
             :output [(str (get programming-language-labels mode)
                           " runtime is not connected yet · runtime ще не підключено · Runtime ist noch nicht verbunden")]
             :error? true))
    (when-not (and (contains? #{"my-lisp" "markdown"} mode) (workspace/native?))
      (render!))))

(defn- cycle-programming-language! []
  (when-let [path (:active-path @state)]
    (let [current (or (get-in @state [:documents path :language-mode]) "text")
          next-mode (next-value programming-languages current)]
      (swap! state assoc-in [:documents path :language-mode] next-mode)
      (render!))))

(defn render! []
  (let [{:keys [language theme root tree open-paths active-path output ast error? sidebar?]} @state 
        app (.getElementById js/document "app") 
        doc (active-doc)
        mode (or (:language-mode doc) "text")
        preview? (or (= mode "markdown") (= mode "mermaid"))
        observatory (:observatory @state)]
    (apply-theme! theme)
    (set! (.-innerHTML app)
      (str "<div class='shell'><header class='topbar'><div class='brand'><button id='menu' class='icon'>☰</button><div class='mark'>λ</div><div><strong>my-idea</strong><small>lightweight programming IDE</small></div></div>"
           "<div class='actions'><button id='language' title='Language'>" (get language-labels language) "</button><button id='theme' title='Theme'>" (get theme-icons theme) " " (get-in messages [language :themes theme]) "</button><button id='open'>" (t :open) "</button><button id='save'>" (t :save) "</button><button id='save-as'>" (t :save-as) "</button>" (when (workspace/native?) "<button id='ecosystem' title='Run ecosystem check'>🔭 Ecosystem</button>") "<button class='run' id='run'>▶ " (t :run) "</button></div></header>"
           (if observatory
             (str "<main class='observatory-workspace'>" (observatory-html observatory) "</main>")
             (str "<main class='workspace" (when-not sidebar? " sidebar-closed") "'><aside class='sidebar'><div class='sidebar-toolbar'><button id='new-file' title='" (t :new-file) "'>&#xFF0B;</button><button id='open-sidebar' title='" (t :open) "'>&#128193;</button></div>" (when root (str "<div class='root'>" (esc root) "</div>")) "<nav>" (workspace/tree-html tree) "</nav></aside>"
           "<section class='center'><div class='tabs'>" (apply str (map #(str "<button class='tab" (when (= % active-path) " active") "' data-tab='" % "'>" (workspace/filename %) (when (get-in @state [:documents % :dirty?]) " •") "<span data-close='" % "'>×</span></button>") open-paths)) "</div><div id='editor'></div></section>"
           "<div class='right'><section class='pane'><div class='pane-head'>" (t :console) "</div><pre" (when error? " class='error'") ">" (esc (str/join "\n" output)) "</pre></section>"
           (if preview?
             (str "<section class='pane preview'><div class='pane-head'>" (t :preview) "</div><div id='preview-content' class='preview-body'></div></section>")
             (str "<section class='pane ast'><div class='pane-head'>" (t :ast) "</div><pre>" (esc ast) "</pre></section>"))
           "</div></main>"))
           "<footer class='status'><span>● " (esc (or active-path "No file")) "</span><button id='programming-language' class='status-language' title='Programming language · Мова програмування · Programmiersprache'>"
           (get programming-language-labels mode)
           "</button><span>Tauri + ClojureScript · UTF-8 · CodeMirror 6</span></footer></div>"))
    (when (and doc (not observatory))
      (editor/mount! (.getElementById js/document "editor") (:contents doc) mode wasm/diagnose
                     #(do (swap! state workspace/update-active %)
                          (when preview? (preview/render! % mode (.getElementById js/document "preview-content")))))
      (when preview?
        (preview/render! (:contents doc) mode (.getElementById js/document "preview-content"))))
    (when-let [el (.getElementById js/document "close-observatory")] (.addEventListener el "click" #(do (swap! state assoc :observatory nil) (render!))))
    (when-let [el (.getElementById js/document "refresh-ecosystem")] (.addEventListener el "click" check-ecosystem!))
    (.addEventListener (.getElementById js/document "open") "click" choose-workspace!)
    (when-let [el (.getElementById js/document "new-file")] (.addEventListener el "click" new-file!))
    (when-let [el (.getElementById js/document "open-sidebar")] (.addEventListener el "click" choose-workspace!))
    (.addEventListener (.getElementById js/document "save") "click" save!)
    (.addEventListener (.getElementById js/document "save-as") "click" save-as!)
    (.addEventListener (.getElementById js/document "run") "click" execute!)
    (when-let [el (.getElementById js/document "ecosystem")] (.addEventListener el "click" check-ecosystem!))
    (.addEventListener (.getElementById js/document "programming-language") "click" cycle-programming-language!)
    (.addEventListener (.getElementById js/document "menu") "click" #(do (swap! state update :sidebar? not) (render!)))
    (.addEventListener (.getElementById js/document "language") "click"
                       #(let [value (next-value languages (:language @state))]
                          (.setItem js/localStorage "my-idea:language" value)
                          (swap! state assoc :language value)
                          (render!)))
    (.addEventListener (.getElementById js/document "theme") "click"
                       #(let [value (next-value themes (:theme @state))]
                          (swap! state assoc :theme value)
                          (render!)))
    (doseq [el (.querySelectorAll js/document "[data-path]")] (.addEventListener el "click" #(open-file! (.. % -currentTarget -dataset -path))))
    (doseq [el (.querySelectorAll js/document "[data-tab]")]
      (.addEventListener el "click" (fn [^js event]
                                       (swap! state assoc :active-path (.. event -currentTarget -dataset -tab))
                                       (persist!)
                                       (render!))))
    (doseq [el (.querySelectorAll js/document "[data-close]")] (.addEventListener el "click" #(do (.stopPropagation %) (swap! state workspace/close-document (.. % -currentTarget -dataset -close)) (persist!) (render!))))))

(defn- restore-native! []
  (when-let [{:keys [root open-paths active-path]} (workspace/restored)]
    (when (and root (workspace/native?))
      (-> (workspace/invoke! "reopen_workspace" {:path root})
          (.then #(do (swap! state assoc :root % :open-paths [] :active-path nil :documents {})
                      (-> (workspace/invoke! "list_workspace" {}) (.then (fn [nodes] (swap! state assoc :tree (js->clj nodes :keywordize-keys true)) (doseq [path open-paths] (open-file! path)) (when active-path (swap! state assoc :active-path active-path)) (render!))))))
          (.catch (fn [_] (.removeItem js/localStorage workspace/storage-key)))))))
(defn ^:export init []
  (render!)
  (restore-native!)
  ;; Load the WASM engine in the background for the web/PWA build.
  ;; The re-render triggered by wasm/load! ensures the first execution
  ;; after loading uses the canonical Rust engine, not the CLJS prototype.
  ;; Завантажуємо WASM-рушій у фоні для веб/PWA збірки.
  ;; Re-render, який тригерить wasm/load!, забезпечує що перше виконання
  ;; після завантаження використовує канонічний Rust-рушій, а не CLJS-прототип.
  ;; Lädt die WASM-Engine im Hintergrund für den Web/PWA-Build.
  (when-not (workspace/native?)
    (wasm/load! render!)))
