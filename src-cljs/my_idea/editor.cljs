(ns my-idea.editor
  (:require [my-idea.language :as idea-language]
            ["@codemirror/autocomplete" :refer [autocompletion completeFromList completionKeymap]]
            ["@codemirror/commands" :refer [defaultKeymap history historyKeymap indentWithTab]]
            ["@codemirror/language" :refer [bracketMatching foldGutter indentOnInput]]
            ["@codemirror/lint" :refer [linter lintGutter]]
            ["@codemirror/state" :refer [EditorState]]
            ["@codemirror/view" :refer [EditorView drawSelection
                                         highlightActiveLine highlightActiveLineGutter
                                         keymap lineNumbers]]
            ["@nextjournal/lang-clojure" :refer [clojure]]))

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
        #js {:label "list" :type "function"}
        #js {:label "vector" :type "function"}
        #js {:label "count" :type "function"}
        #js {:label "pi" :type "constant"}]))

(defn- diagnostics [view]
  (let [source (.. view -state -doc toString)]
    (try
      (idea-language/parse-program source)
      #js []
      (catch :default error
        #js [#js {:from 0
                  :to (min 1 (count source))
                  :severity "error"
                  :message (.-message error)}]))))

(def theme
  (.theme EditorView
          #js {"&" #js {:height "100%" :backgroundColor "#0d1424" :color "#dbeafe"}
               ".cm-scroller" #js {:fontFamily "Cascadia Code, Fira Code, monospace"
                                    :fontSize "15px" :lineHeight "1.65"}
               ".cm-content" #js {:padding "14px 0" :caretColor "#38bdf8"}
               ".cm-gutters" #js {:backgroundColor "#0b1220" :color "#64748b"
                                   :borderRight "1px solid #243047"}
               ".cm-activeLine, .cm-activeLineGutter" #js {:backgroundColor "#1e293b80"}
               ".cm-selectionBackground, &.cm-focused .cm-selectionBackground" #js {:backgroundColor "#334b78"}
               "&.cm-focused" #js {:outline "none"}}
          #js {:dark true}))

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

(defn mount!
  "Mount the programming editor. The evaluator is only one optional consumer."
  [parent source-text on-change]
  (when-let [^js old-view @view*]
    (.destroy old-view))
  (let [state (.create EditorState
                       #js {:doc source-text
                            :extensions
                            #js [(lineNumbers) (highlightActiveLineGutter) (foldGutter)
                                 (history) (drawSelection) (indentOnInput)
                                 (bracketMatching) (highlightActiveLine)
                                 (clojure)
                                 (.of keymap
                                      (.concat #js [indentWithTab]
                                               defaultKeymap historyKeymap completionKeymap))
                                 (autocompletion #js {:override #js [completions]})
                                 (lintGutter) (linter diagnostics)
                                 theme
                                 (.of (.-updateListener EditorView)
                                      (fn [^js update]
                                        (when (.-docChanged update)
                                          (on-change (.. update -state -doc toString)))))]})
        view (EditorView. #js {:state state :parent parent})]
    (reset! view* view)
    view))
