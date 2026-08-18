(ns my-idea.state
  "The app-wide state atom, split out of core.cljs so other namespaces
  (commands, views) can eventually require it directly instead of core.
  Deliberately just the atom + its initial value — the many functions
  that read/mutate it stay in core.cljs for now (IDEA-CORE-CLJS-SPLIT-STATE
  scopes this as a real, incremental cut, not a one-shot mechanical
  rename of every call site).")

(defonce state (atom {:language (or (.getItem js/localStorage "my-idea:language") "uk")
                       :theme (or (.getItem js/localStorage "my-idea:theme") "auto")
                       :root nil
                       :tree []
                       :open-paths []
                       :active-path nil
                       :documents {}
                       :output ["Ready · Готово · Bereit"] :ast "[]" :error? false :sidebar? true
                       :ecosystem nil :selected-requirement nil :knowledge-graph nil}))

(defn active-doc
  "Returns the currently active document from state, or nil."
  []
  (get-in @state [:documents (:active-path @state)]))
