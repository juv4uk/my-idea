(ns my-idea.editor
  (:require ["@codemirror/autocomplete" :refer [autocompletion completeFromList completionKeymap]]
            ["@codemirror/commands" :refer [defaultKeymap history historyKeymap indentWithTab]]
            ["@codemirror/language" :refer [bracketMatching foldGutter indentOnInput]]
            ["@codemirror/state" :refer [EditorState]]
            ["@codemirror/view" :refer [EditorView drawSelection
                                         highlightActiveLine highlightActiveLineGutter
                                         keymap lineNumbers]]
            ["@codemirror/lang-rust" :refer [rust]]
            ["@codemirror/lang-markdown" :refer [markdown]]
            ["codemirror-lang-mermaid" :refer [mermaid]]
            ["@nextjournal/lang-clojure" :refer [clojure]]
            ["@codemirror/lint" :refer [linter lintGutter]]))

;; One CodeMirror instance is shared by the workspace.
;; Один екземпляр CodeMirror обслуговує робочу область.
;; Eine CodeMirror-Instanz versorgt den Arbeitsbereich.
(defonce view* (atom nil))

(def completions
  (completeFromList
   #js [#js {:label "def" :type "keyword" :detail "Bind a value"}
        #js {:label "if" :type "keyword" :detail "Conditional expression"}
        #js {:label "println" :type "function" :detail "Print to the IDE console"}
        #js {:label "quote" :type "keyword"}
        #js {:label "cond" :type "keyword" :detail "McCarthy conditional"}
        #js {:label "atom" :type "function" :detail "Test whether a value is an atom"}
        #js {:label "eq" :type "function" :detail "Compare two atoms"}
        #js {:label "car" :type "function" :detail "First item of a list"}
        #js {:label "cdr" :type "function" :detail "Rest of a list"}
        #js {:label "cons" :type "function" :detail "Prepend an item to a list"}
        #js {:label "list" :type "function"}
        #js {:label "vector" :type "function"}
        #js {:label "count" :type "function"}
        #js {:label "pi" :type "constant"}]))

(defn- editor-theme []
  (let [theme (or (.. js/document -documentElement -dataset -theme) "auto")
        dark? (or (contains? #{"dark" "signal" "amber" "forest"} theme)
                  (and (= theme "auto") (.-matches (js/window.matchMedia "(prefers-color-scheme: dark)"))))
        colors (cond
                 (= theme "signal")
                 {:background "#0d1424" :text "#dbeafe" :gutter "#0b1220"
                  :muted "#64748b" :line "#243047" :active "#1e293b" :selection "#334b78"}
                 (= theme "amber")
                 {:background "#1c1712" :text "#e3d5bd" :gutter "#17120e"
                  :muted "#8f806b" :line "#3b3024" :active "#292117" :selection "#55432c"}
                 (= theme "forest")
                 {:background "#111a17" :text "#d2ddd5" :gutter "#0d1512"
                  :muted "#76867b" :line "#293a31" :active "#1b2821" :selection "#304c3d"}
                 (= theme "sepia")
                 {:background "#eee5d2" :text "#463f35" :gutter "#e3d7bf"
                  :muted "#817565" :line "#cfc0a4" :active "#e6dbc5" :selection "#c8d6cf"}
                 dark?
                 {:background "#151a22" :text "#d8d5cc" :gutter "#11161d"
                  :muted "#777f89" :line "#222a35" :active "#202934" :selection "#30445a"}
                 :else
                 {:background "#f3f0e8" :text "#343a40" :gutter "#ebe7dd"
                  :muted "#7b8083" :line "#d7d2c7" :active "#e7edf0" :selection "#bfd8df"})]
    (.theme EditorView
          #js {"&" #js {:height "100%" :backgroundColor (:background colors) :color (:text colors)}
               ".cm-scroller" #js {:fontFamily "Cascadia Code, Fira Code, monospace"
                                    :fontSize "15px" :lineHeight "1.65"}
               ".cm-content" #js {:padding "14px 0" :caretColor "#3b8998"}
               ".cm-gutters" #js {:backgroundColor (:gutter colors) :color (:muted colors)
                                   :borderRight (str "1px solid " (:line colors))}
               ".cm-activeLine, .cm-activeLineGutter" #js {:backgroundColor (:active colors)}
               ".cm-selectionBackground, &.cm-focused .cm-selectionBackground" #js {:backgroundColor (:selection colors)}
               "&.cm-focused" #js {:outline "none"}}
          #js {:dark dark?})))

(defn source []
  (if-let [view @view*]
    (.. view -state -doc toString)
    ""))

(defn set-source! [text]
  (when-let [view @view*]
    (.dispatch view
               #js {:changes #js {:from 0
                                   :to (.. view -state -doc -length)
                                   :insert text}})))

(defn- language-extensions [mode]
  (case mode
    "rust" #js [(rust)]
    "markdown" #js [(markdown)]
    "mermaid" #js [(mermaid)]
    "text" #js []
    #js [(clojure) (autocompletion #js {:override #js [completions]})]))

(defn mount!
  "Mount the programming editor. The evaluator is only one optional consumer."
  [parent source-text mode diagnose-fn on-change]
  (when-let [^js old-view @view*]
    (.destroy old-view))
  (let [state (.create EditorState
                       #js {:doc source-text
                            :extensions
                            #js [(lineNumbers) (highlightActiveLineGutter) (foldGutter)
                                 (history) (drawSelection) (indentOnInput)
                                 (bracketMatching) (highlightActiveLine)
                                 (language-extensions mode)
                                 (lintGutter)
                                 (linter (fn [view] (diagnose-fn (.. view -state -doc toString) mode)))
                                 (.of keymap
                                      (.concat #js [indentWithTab]
                                               defaultKeymap historyKeymap completionKeymap))
                                 (editor-theme)
                                 (.of (.-updateListener EditorView)
                                      (fn [^js update]
                                        (when (.-docChanged update)
                                          (on-change (.. update -state -doc toString)))))]})
        view (EditorView. #js {:state state :parent parent})]
    (reset! view* view)
    view))
