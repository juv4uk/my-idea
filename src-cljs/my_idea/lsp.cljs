(ns my-idea.lsp
  "Thin CodeMirror adapter for the authoritative native language servers."
  (:require [clojure.string :as str]
            [my-idea.workspace :as workspace]))

(defonce diagnostics* (atom {}))
(defonce versions* (atom {}))
(defonce listening?* (atom false))
(defonce refresh-fn* (atom nil))
(defonce listener-ready* (atom nil))

(defn set-refresh! [f] (reset! refresh-fn* f))

(defn supported? [mode] (contains? #{"my-lisp" "rust"} mode))
(defn- command-prefix [mode] (if (= mode "rust") "rust_lsp_" "wsm_lsp_"))

(defn- event-listen []
  (some-> (aget js/window "__TAURI__") (aget "event") (aget "listen")))

(defn init! []
  (when (and (workspace/native?) (not @listening?*))
    (when-let [listen (event-listen)]
      (reset! listening?* true)
      (reset! listener-ready*
              (listen "lsp-message"
                      (fn [^js event]
                        (let [message (js->clj (.-payload event) :keywordize-keys true)]
                          (when (= (:method message) "textDocument/publishDiagnostics")
                            (let [params (:params message)]
                              (swap! diagnostics* assoc
                                     (try (js/decodeURI (:uri params)) (catch :default _ (:uri params)))
                                     (:diagnostics params))
                              (when @refresh-fn* (@refresh-fn*)))))))))))

(defn- after-listener [f]
  (if-let [ready @listener-ready*]
    (.then ready f)
    (f)))

(defn- matching-diagnostics [path]
  (or (some (fn [[uri diagnostics]]
              (when (or (str/ends-with? uri (str "/" path))
                        (str/ends-with? uri path))
                diagnostics))
            @diagnostics*)
      []))

(defn- position-offset [doc {:keys [line character]}]
  (let [line-info (.line doc (min (inc (or line 0)) (.-lines doc)))]
    (min (.-to line-info) (+ (.-from line-info) (or character 0)))))

(defn diagnostics [path view]
  (let [doc (.. view -state -doc)]
    (clj->js
     (map (fn [diagnostic]
            {:from (position-offset doc (get-in diagnostic [:range :start]))
             :to (position-offset doc (get-in diagnostic [:range :end]))
             :severity (case (:severity diagnostic) 1 "error" 2 "warning" 3 "info" 4 "info" "error")
             :message (:message diagnostic)
             :source (or (:source diagnostic) "WsmLS")})
          (matching-diagnostics path)))))

(defn open! [mode path text]
  (let [key [mode path]]
    (when (and (workspace/native?) (supported? mode) path (not (contains? @versions* key)))
      (swap! versions* assoc key 1)
      (-> (after-listener #(workspace/invoke! (str (command-prefix mode) "open")
                                               {:path path :text text :version 1}))
          (.catch #(js/console.warn "LSP open failed" %))))))

(defn change! [mode path text]
  (let [key [mode path]]
    (when (and (workspace/native?) (contains? @versions* key))
      (let [version (get (swap! versions* update key inc) key)]
        (-> (workspace/invoke! (str (command-prefix mode) "change")
                               {:path path :text text :version version})
            (.catch #(js/console.warn "LSP change failed" %)))))))

(defn close! [mode path]
  (let [key [mode path]]
    (when (and (workspace/native?) (contains? @versions* key))
      (swap! versions* dissoc key)
      (-> (workspace/invoke! (str (command-prefix mode) "close") {:path path})
          (.catch #(js/console.warn "LSP close failed" %))))))

(defn completions [mode path]
  (fn [^js context]
    (let [position (.-pos context)
          line-info (.. context -state -doc (lineAt position))
          line (dec (.-number line-info))
          character (- position (.-from line-info))
          word (.matchBefore context #"[A-Za-z0-9_?!+*/<>=-]*")]
      (-> (workspace/invoke! (str (command-prefix mode) "completion")
                             {:path path :line line :character character})
          (.then (fn [response]
                   (let [result (aget response "result")
                         items (if (array? result) result (some-> result (aget "items")))]
                     #js {:from (if word (.-from word) position)
                          :options (or items #js [])})))
          (.catch (fn [_] #js {:from position :options #js []}))))))

(defn- hover-content [contents]
  (cond
    (string? contents) contents
    (map? contents) (or (:value contents) "")
    (sequential? contents) (str/join "\n\n" (map hover-content contents))
    :else ""))

(defn hover [mode path]
  (fn [^js view position _side]
    (let [line-info (.. view -state -doc (lineAt position))
          line (dec (.-number line-info))
          character (- position (.-from line-info))]
      (-> (workspace/invoke! (str (command-prefix mode) "hover")
                             {:path path :line line :character character})
          (.then (fn [response]
                   (when-let [result (some-> response (aget "result"))]
                     (let [text (hover-content (:contents (js->clj result :keywordize-keys true)))]
                       (when (seq text)
                         #js {:pos position
                              :above true
                              :create (fn []
                                        (let [dom (.createElement js/document "div")]
                                          (set! (.-className dom) "cm-lsp-hover")
                                          (set! (.-textContent dom) text)
                                          #js {:dom dom}))})))))
          (.catch (fn [_] nil))))))
