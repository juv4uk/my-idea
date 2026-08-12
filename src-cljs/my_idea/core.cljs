(ns my-idea.core
  (:require [clojure.string :as str]
            [my-idea.eco-view :as eco-view]
            [my-idea.editor :as editor]
            [my-idea.i18n :as i18n]
            [my-idea.preview :as preview]
            [my-idea.state :refer [state]]
            [my-idea.util :as util]
            [my-idea.wasm :as wasm] [my-idea.workspace :as workspace]))

(def demo-source
  "; my-lisp · Rust/CLJS shared contract · спільний контракт · gemeinsamer Vertrag\n(def greeting \"Hello · Привіт · Hallo\")\n(def second (lambda (values) (car (cdr values))))\n(cons greeting (cons (second '(radio antenna)) '()))")

(def markdown-demo
  "# my-lisp literate document\n\nThis is a standard markdown document that mixes prose and code.\n\n```my-lisp\n;; This code block is extracted and evaluated by the engine!\n(def text \"Hello from Literate my-lisp!\")\ntext\n```\n")

(def mermaid-demo
  "graph TD\n    A[Welcome] -->|Evaluate| B(my-lisp)\n    B --> C{Platform}\n    C -->|Desktop| D[Tauri]\n    C -->|Web| E[WASM]\n")

(defn- t [key] (i18n/t (:language @state) key))
(defn- esc [x] (util/esc x))
(defn- active-doc [] (get-in @state [:documents (:active-path @state)]))
(defn- next-value [values current] (util/next-value values current))
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
                (swap! state assoc :ecosystem status :selected-requirement nil
                       :output ["Ecosystem check complete · Перевірку екосистеми завершено"] :error? false)
                (render!)))
      (.catch #(do (swap! state assoc :output [(str %)] :error? true) (render!)))))

(defn- ask-oracle! []
  (when-let [doc (active-doc)]
    (swap! state assoc :ecosystem nil :output ["Asking my-lisp oracle (127.0.0.1:9999)… · Питаємо оракула my-lisp…"] :error? false)
    (render!)
    (-> (workspace/invoke! "oracle_query" {:source (:contents doc) :op "eval"})
        (.then #(let [{:keys [status kind message raw]} (js->clj % :keywordize-keys true)
                      ok? (= status "ok")
                      lines (if ok?
                              [raw]
                              [(str "oracle: " status (when kind (str " (" kind ")")))
                               (or message raw)])]
                  (swap! state assoc :output lines :error? (not ok?))
                  (render!)))
        (.catch #(let [message (str %)
                       not-running? (str/includes? message "could not connect")]
                   (swap! state assoc
                          :output [(if not-running?
                                     (str "Oracle not reachable on 127.0.0.1:9999 · Оракул недоступний на 127.0.0.1:9999\n"
                                          "Start it in the my-lisp repo · Запустіть його в репо my-lisp:\n"
                                          "  cargo run -p my-lisp-cli -- --tcp --protocol=sexpr\n\n" message)
                                     message)]
                          :error? true)
                   (render!))))))

(defn- swarm-status! []
  (swap! state assoc :ecosystem nil :output ["Asking my-idea's swarm-node (127.0.0.1:9104)… · Питаємо swarm-node…"] :error? false)
  (render!)
  (-> (workspace/invoke! "swarm_status" {})
      (.then #(do (swap! state assoc :output [%] :error? false) (render!)))
      (.catch #(let [message (str %)
                     not-running? (str/includes? message "could not connect")]
                 (swap! state assoc
                        :output [(if not-running?
                                   (str "swarm-node not reachable on 127.0.0.1:9104 · swarm-node недоступний\n"
                                        "Start it in the my-lisp repo · Запустіть у репо my-lisp:\n"
                                        "  ./target/debug/swarm-node --port 9104 --node-id my-idea-1 --project my-idea "
                                        "--data-dir ~/.swarm-node/my-idea-1 --connect 127.0.0.1:9101\n\n" message)
                                   message)]
                        :error? true)
                 (render!)))))

(defn- settle [p]
  (-> p (.then (fn [v] #js {:ok true :value v}))
      (.catch (fn [e] #js {:ok false :value (str e)}))))

(defn- compare-with-oracle! []
  (when-let [doc (active-doc)]
    (let [source (:contents doc)]
      (swap! state assoc :ecosystem nil
             :output ["Comparing local engine vs my-lisp oracle… · Порівнюємо локальний рушій з оракулом…"]
             :error? false)
      (render!)
      (-> (.all js/Promise #js [(settle (workspace/invoke! "evaluate_my_lisp" {:source source :mode "my-lisp"}))
                                 (settle (workspace/invoke! "oracle_query" {:source source :op "eval"}))])
          (.then (fn [results]
                   (let [[local oracle] (js->clj results :keywordize-keys true)
                         local-ok? (:ok local)
                         local-value (when local-ok? (get-in local [:value :value]))
                         oracle-invoke-ok? (:ok oracle)
                         oracle-resp (when oracle-invoke-ok? (:value oracle))
                         oracle-success? (and oracle-invoke-ok? (= (:status oracle-resp) "ok"))
                         oracle-raw (:raw oracle-resp)
                         match? (and local-ok? oracle-success?
                                     (str/includes? oracle-raw (str "(value " local-value ")")))
                         lines [(if local-ok?
                                  (str "local (Rust): " local-value)
                                  (str "local (Rust): ERROR " (:value local)))
                                (cond
                                  oracle-success? (str "oracle (my-lisp TCP): " oracle-raw)
                                  oracle-invoke-ok? (str "oracle (my-lisp TCP): error " (:status oracle-resp)
                                                          " — " (or (:message oracle-resp) (:raw oracle-resp)))
                                  :else (str "oracle (my-lisp TCP): " (:value oracle)))
                                (if (and local-ok? oracle-success?)
                                  (if match? "✓ agreement · збіг" "✗ MISMATCH · розбіжність")
                                  "— comparison incomplete (one side failed) · порівняння неповне")]]
                     (swap! state assoc :output lines :error? (not (and local-ok? oracle-success? match?)))
                     (render!))))))))

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
             :output [(str (get i18n/programming-language-labels mode)
                           " runtime is not connected yet · runtime ще не підключено · Runtime ist noch nicht verbunden")]
             :error? true))
    (when-not (and (contains? #{"my-lisp" "markdown"} mode) (workspace/native?))
      (render!))))

(defn- cycle-programming-language! []
  (when-let [path (:active-path @state)]
    (let [current (or (get-in @state [:documents path :language-mode]) "text")
          next-mode (next-value i18n/programming-languages current)]
      (swap! state assoc-in [:documents path :language-mode] next-mode)
      (render!))))

(defn render! []
  (let [{:keys [language theme root tree open-paths active-path output ast error? sidebar? ecosystem selected-requirement]} @state
        app (.getElementById js/document "app")
        doc (active-doc)
        mode (or (:language-mode doc) "text")
        preview? (or (= mode "markdown") (= mode "mermaid"))]
    (apply-theme! theme)
    (set! (.-innerHTML app)
      (str "<div class='shell'><header class='topbar'><div class='brand'><button id='menu' class='icon'>☰</button><div class='mark'>λ</div><div><strong>my-idea</strong><small>lightweight programming IDE</small></div></div>"
           "<div class='actions'><button id='language' title='Language'>" (get i18n/language-labels language) "</button><button id='theme' title='Theme'>" (get i18n/theme-icons theme) " " (get-in i18n/messages [language :themes theme]) "</button><button id='open'>" (t :open) "</button><button id='save'>" (t :save) "</button><button id='save-as'>" (t :save-as) "</button>" (when (workspace/native?) "<button id='ecosystem' title='Run ecosystem check'>🔭 Ecosystem</button><button id='oracle' title='Ask the live my-lisp TCP oracle (127.0.0.1:9999)'>🔮 Oracle</button><button id='compare' title='Compare the embedded Rust engine against the live my-lisp TCP oracle'>⚖ Compare</button><button id='swarm' title='Show this agent&#39;s swarm-node status (127.0.0.1:9104)'>🐝 Swarm</button>") "<button class='run' id='run'>▶ " (t :run) "</button></div></header>"
           "<main class='workspace" (when-not sidebar? " sidebar-closed") "'><aside class='sidebar'><div class='sidebar-toolbar'><button id='new-file' title='" (t :new-file) "'>&#xFF0B;</button><button id='open-sidebar' title='" (t :open) "'>&#128193;</button></div>" (when root (str "<div class='root'>" (esc root) "</div>")) "<nav>" (workspace/tree-html tree) "</nav></aside>"
           "<section class='center'><div class='tabs'>" (apply str (map #(str "<button class='tab" (when (= % active-path) " active") "' data-tab='" % "'>" (workspace/filename %) (when (get-in @state [:documents % :dirty?]) " •") "<span data-close='" % "'>×</span></button>") open-paths)) "</div><div id='editor'></div></section>"
           "<div class='right'><section class='pane'><div class='pane-head'>" (t :console) "</div><pre" (when error? " class='error'") ">" (esc (str/join "\n" output)) "</pre></section>"
           (cond
             ecosystem (str "<section class='pane eco-pane'><div class='pane-head'>" (t :ecosystem) "</div>" (eco-view/ecosystem-html ecosystem selected-requirement) "</section>")
             preview? (str "<section class='pane preview'><div class='pane-head'>" (t :preview) "</div><div id='preview-content' class='preview-body'></div></section>")
             :else (str "<section class='pane ast'><div class='pane-head'>" (t :ast) "</div><pre>" (esc ast) "</pre></section>"))
           "</div></main>"
           "<footer class='status'><span>● " (esc (or active-path "No file")) "</span><button id='programming-language' class='status-language' title='Programming language · Мова програмування · Programmiersprache'>"
           (get i18n/programming-language-labels mode)
           "</button><span>Tauri + ClojureScript · UTF-8 · CodeMirror 6</span></footer></div>"))
    (when doc 
      (editor/mount! (.getElementById js/document "editor") (:contents doc) mode wasm/diagnose
                     #(do (swap! state workspace/update-active %)
                          (when preview? (preview/render! % mode (.getElementById js/document "preview-content")))))
      (when preview?
        (preview/render! (:contents doc) mode (.getElementById js/document "preview-content"))))
    (.addEventListener (.getElementById js/document "open") "click" choose-workspace!)
    (.addEventListener (.getElementById js/document "new-file") "click" new-file!)
    (.addEventListener (.getElementById js/document "open-sidebar") "click" choose-workspace!)
    (.addEventListener (.getElementById js/document "save") "click" save!)
    (.addEventListener (.getElementById js/document "save-as") "click" save-as!)
    (.addEventListener (.getElementById js/document "run") "click" execute!)
    (when-let [el (.getElementById js/document "ecosystem")] (.addEventListener el "click" check-ecosystem!))
    (when-let [el (.getElementById js/document "oracle")] (.addEventListener el "click" ask-oracle!))
    (when-let [el (.getElementById js/document "compare")] (.addEventListener el "click" compare-with-oracle!))
    (when-let [el (.getElementById js/document "swarm")] (.addEventListener el "click" swarm-status!))
    (when-let [el (.getElementById js/document "eco-back")]
      (.addEventListener el "click" #(do (swap! state assoc :selected-requirement nil) (render!))))
    (doseq [el (.querySelectorAll js/document "[data-req]")]
      (.addEventListener el "click" #(do (swap! state assoc :selected-requirement (.. % -currentTarget -dataset -req)) (render!))))
    (.addEventListener (.getElementById js/document "programming-language") "click" cycle-programming-language!)
    (.addEventListener (.getElementById js/document "menu") "click" #(do (swap! state update :sidebar? not) (render!)))
    (.addEventListener (.getElementById js/document "language") "click"
                       #(let [value (next-value i18n/languages (:language @state))]
                          (.setItem js/localStorage "my-idea:language" value)
                          (swap! state assoc :language value)
                          (render!)))
    (.addEventListener (.getElementById js/document "theme") "click"
                       #(let [value (next-value i18n/themes (:theme @state))]
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
