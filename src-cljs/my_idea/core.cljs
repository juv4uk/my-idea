(ns my-idea.core
  (:require [cljs.pprint :as pprint]
            [clojure.string :as str]
            [my-idea.editor :as editor]
            [my-idea.language :as language]))

(defonce state
  (atom {:language (or (.getItem js/localStorage "my-idea:language") "uk")
         :source (or (.getItem js/localStorage "my-idea:source")
                     "; my-idea programming workspace\n(def radius 7)\n(def area (* pi radius radius))\n(println \"area =\" area)\narea")
         :output ["Ready · Готово · Bereit"] :ast "[]" :error? false}))

(def messages
  {"en" {:tagline "lightweight programming IDE" :run "Run" :example "Example"
         :editor "Editor" :console "Console" :ast "Language Lab / AST" :status "Local, offline workspace"}
   "uk" {:tagline "легка IDE для програмування" :run "Запустити" :example "Приклад"
         :editor "Редактор" :console "Консоль" :ast "Лабораторія мов / AST" :status "Локальне офлайн-середовище"}
   "de" {:tagline "leichtgewichtige Programmier-IDE" :run "Starten" :example "Beispiel"
         :editor "Editor" :console "Konsole" :ast "Sprachlabor / AST" :status "Lokaler Offline-Arbeitsbereich"}})

(defn- t [key] (get-in messages [(:language @state) key]))
(defn- escape-html [value]
  (-> (str value) (str/replace "&" "&amp;") (str/replace "<" "&lt;")
      (str/replace ">" "&gt;") (str/replace "\"" "&quot;")))

(declare render!)

(defn- remember-source! [source]
  (.setItem js/localStorage "my-idea:source" source)
  (swap! state assoc :source source))

(defn- execute! []
  (let [source (editor/source)]
    (remember-source! source)
    (try
      (let [{:keys [value output forms]} (language/run-program source)]
        (swap! state assoc :output (conj (vec output) (str "=> " (pr-str value)))
               :ast (with-out-str (pprint/pprint forms)) :error? false))
      (catch :default error
        (swap! state assoc :output [(.-message error)] :ast "Parse/evaluation stopped" :error? true)))
    (render!)))

(def example-source
  "; CodeMirror 6 + our small Lisp laboratory\n(def greeting \"Hello · Привіт · Hallo\")\n(def power-mw 500)\n(println greeting)\n(println \"power =\" power-mw \"mW\")\n(if (< power-mw 1000) \"QRPp\" \"QRO\")")

(defn render! []
  (let [{:keys [language source output ast error?]} @state
        app (.getElementById js/document "app")]
    (set! (.-innerHTML app)
          (str "<div class='shell'><header class='topbar'><div class='brand'><div class='mark'>λ</div><div><strong>my-idea</strong><small>" (t :tagline) " · Tauri + ClojureScript</small></div></div>"
               "<div class='actions'><select id='language' aria-label='Language'><option value='en'" (when (= language "en") " selected") ">EN</option><option value='uk'" (when (= language "uk") " selected") ">UA</option><option value='de'" (when (= language "de") " selected") ">DE</option></select><button id='example'>" (t :example) "</button><button class='run' id='run'>▶ " (t :run) "</button></div></header>"
               "<main class='workspace'><section class='pane'><div class='pane-head'><span>" (t :editor) "</span><span>main.clj · CodeMirror 6</span></div><div id='editor'></div></section>"
               "<div class='right'><section class='pane'><div class='pane-head'><span>" (t :console) "</span><span>embedded Lisp</span></div><pre" (when error? " class='error'") ">" (escape-html (str/join "\n" output)) "</pre></section>"
               "<section class='pane ast'><div class='pane-head'><span>" (t :ast) "</span><span>EDN</span></div><pre>" (escape-html ast) "</pre></section></div></main>"
               "<footer class='status'><span>● " (t :status) "</span><span>UTF-8 · Clojure syntax · v0.1.0</span></footer></div>"))
    (editor/mount! (.getElementById js/document "editor") source remember-source!)
    (.addEventListener (.getElementById js/document "run") "click" execute!)
    (.addEventListener (.getElementById js/document "example") "click"
                       #(do (swap! state assoc :source example-source :output ["Example loaded."] :ast "[]") (render!)))
    (.addEventListener (.getElementById js/document "language") "change"
                       #(let [value (.. % -target -value)]
                          (.setItem js/localStorage "my-idea:language" value)
                          (swap! state assoc :language value :source (editor/source))
                          (render!)))))

(defn ^:export init [] (render!))
