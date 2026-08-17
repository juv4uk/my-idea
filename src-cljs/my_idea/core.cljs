(ns my-idea.core
  (:require [clojure.string :as str]
            [my-idea.commands :as cmd]
            [my-idea.eco-view :as eco-view]
            [my-idea.editor :as editor]
            [my-idea.i18n :as i18n]
            [my-idea.preview :as preview]
            [my-idea.state :as state :refer [state active-doc]]
            [my-idea.util :as util]
            [my-idea.wasm :as wasm]
            [my-idea.workspace :as workspace]))

(def demo-source
  "; my-lisp · Rust/CLJS shared contract · спільний контракт · gemeinsamer Vertrag\n(def greeting \"Hello · Привіт · Hallo\")\n(def second (lambda (values) (car (cdr values))))\n(cons greeting (cons (second '(radio antenna)) '()))")

(def markdown-demo
  "# my-lisp literate document\n\nThis is a standard markdown document that mixes prose and code.\n\n```my-lisp\n;; This code block is extracted and evaluated by the engine!\n(def text \"Hello from Literate my-lisp!\")\ntext\n```\n")

(def mermaid-demo
  "graph TD\n    A[Welcome] -->|Evaluate| B(my-lisp)\n    B --> C{Platform}\n    C -->|Desktop| D[Tauri]\n    C -->|Web| E[WASM]\n")

(defn- t [key] (i18n/t (:language @state) key))
(defn- esc [x] (util/esc x))
(defn- next-value [values current] (util/next-value values current))
(defn- apply-theme! [theme]
  (set! (.. js/document -documentElement -dataset -theme) theme)
  (.setItem js/localStorage "my-idea:theme" theme))

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
    (.addEventListener (.getElementById js/document "open") "click" cmd/choose-workspace!)
    (.addEventListener (.getElementById js/document "new-file") "click" cmd/new-file!)
    (.addEventListener (.getElementById js/document "open-sidebar") "click" cmd/choose-workspace!)
    (.addEventListener (.getElementById js/document "save") "click" cmd/save!)
    (.addEventListener (.getElementById js/document "save-as") "click" cmd/save-as!)
    (.addEventListener (.getElementById js/document "run") "click" cmd/execute!)
    (when-let [el (.getElementById js/document "ecosystem")] (.addEventListener el "click" cmd/check-ecosystem!))
    (when-let [el (.getElementById js/document "oracle")] (.addEventListener el "click" cmd/ask-oracle!))
    (when-let [el (.getElementById js/document "compare")] (.addEventListener el "click" cmd/compare-with-oracle!))
    (when-let [el (.getElementById js/document "swarm")] (.addEventListener el "click" cmd/swarm-status!))
    (when-let [el (.getElementById js/document "eco-run-check")] (.addEventListener el "click" cmd/check-ecosystem!))
    (when-let [el (.getElementById js/document "eco-back")]
      (.addEventListener el "click" #(do (swap! state assoc :selected-requirement nil) (render!))))
    (doseq [el (.querySelectorAll js/document "[data-req]")]
      (.addEventListener el "click" #(do (swap! state assoc :selected-requirement (.. % -currentTarget -dataset -req)) (render!))))
    (.addEventListener (.getElementById js/document "programming-language") "click" cmd/cycle-programming-language!)
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
    (doseq [el (.querySelectorAll js/document "[data-path]")] (.addEventListener el "click" #(cmd/open-file! (.. % -currentTarget -dataset -path))))
    (doseq [el (.querySelectorAll js/document "[data-tab]")]
      (.addEventListener el "click" (fn [^js event]
                                       (swap! state assoc :active-path (.. event -currentTarget -dataset -tab))
                                       (cmd/persist!)
                                       (render!))))
    (doseq [el (.querySelectorAll js/document "[data-close]")] (.addEventListener el "click" #(do (.stopPropagation %) (swap! state workspace/close-document (.. % -currentTarget -dataset -close)) (cmd/persist!) (render!))))))

(defn ^:export init []
  (cmd/set-render! render!)
  (render!)
  (cmd/restore-native!)
  (when-not (workspace/native?)
    (wasm/load! render!)))
