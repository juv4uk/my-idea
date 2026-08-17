(ns my-idea.commands
  "Command/action functions extracted from core.cljs — the app's
  controller layer between the UI events and the backend (Tauri IPC,
  WASM, workspace). Each function mutates state and calls render!.

  render! is wired via set-render! (called once from core/init)
  to avoid a circular namespace require."
  (:require [clojure.string :as str]
            [my-idea.editor :as editor]
            [my-idea.i18n :as i18n]
            [my-idea.state :as state :refer [state active-doc]]
            [my-idea.util :as util]
            [my-idea.wasm :as wasm]
            [my-idea.workspace :as workspace]))

;; ---- render callback (set once by core/init) ----

(defonce render-fn (atom nil))

(defn set-render! "Wire the render callback from core.cljs." [f] (reset! render-fn f))
(defn- render! [] (@render-fn))

;; ---- helpers ----

(defn t [key] (i18n/t (:language @state) key))

(defn settle [p]
  (-> p (.then (fn [v] #js {:ok true :value v}))
      (.catch (fn [e] #js {:ok false :value (str e)}))))

;; ---- workspace commands ----

(defn persist! [] (workspace/remember! @state))

(defn refresh-tree! []
  (-> (workspace/invoke! "list_workspace" {})
      (.then #(do (swap! state assoc :tree (js->clj % :keywordize-keys true)) (persist!) (render!)))
      (.catch #(do (swap! state assoc :output [(str %)] :error? true) (render!)))))

(defn choose-browser-workspace! []
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

(defn open-file! [path]
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
        (do (swap! state assoc
                   :output [(case (:language @state)
                              "uk" "📁 Після перезавантаження сторінки потрібно знову відкрити папку (кнопка 📁 зліва)"
                              "de" "📁 Nach dem Neuladen muss der Ordner erneut geöffnet werden (Schaltfläche 📁 links)"
                              "📁 After page reload, re-open the folder using the 📁 button on the left")]
                   :error? false)
            (render!))))))

(defn new-file! []
  (let [name (js/prompt (case (:language @state)
                          "uk" "Назва файлу:"
                          "de" "Dateiname:"
                          "File name:") "untitled.my")]
    (when (and name (not (str/blank? name)))
      (let [path (if (:root @state) name (str/trim name))]
        (swap! state workspace/open-document path "")
        (persist!)
        (render!)))))

(defn choose-workspace! []
  (if (workspace/native?)
    (-> (workspace/invoke! "choose_workspace" {})
        (.then #(when % (swap! state assoc :root % :tree [] :open-paths [] :active-path nil :documents {}) (refresh-tree!))))
    (choose-browser-workspace!)))

(defn save! []
  (when-let [path (:active-path @state)]
    (let [contents (editor/source)]
      (if (and (workspace/native?) (:root @state))
        (-> (workspace/invoke! "save_workspace_file" {:path path :contents contents})
            (.then #(do (swap! state update-in [:documents path] merge {:contents contents :saved contents :dirty? false}) (render!)))
            (.catch #(do (swap! state assoc :output [(str %)] :error? true) (render!))))
        (do (workspace/download! path contents)
            (swap! state update-in [:documents path] merge {:contents contents :saved contents :dirty? false})
            (render!))))))

(defn save-as! []
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

;; ---- eval / oracle / ecosystem ----

(defn handle-eval-result! [result]
  (let [{:keys [value output ast engine]} (js->clj result :keywordize-keys true)]
    (swap! state assoc :output (into [(str engine)] (conj (vec output) (str "=> " value)))
           :ast ast :error? false)
    (render!)))

(defn handle-eval-error! [error]
  (swap! state assoc :output [(str error)] :ast "Parse/evaluation stopped" :error? true)
  (render!))

(defn check-ecosystem! []
  (-> (workspace/invoke! "ecosystem_status" {})
      (.then #(let [status (js->clj % :keywordize-keys true)]
                (swap! state assoc :ecosystem status :selected-requirement nil
                       :output ["Ecosystem check complete · Перевірку екосистеми завершено"] :error? false)
                (render!)))
      (.catch #(do (swap! state assoc :output [(str %)] :error? true) (render!)))))

(defn ask-oracle! []
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

(defn swarm-status! []
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

(defn compare-with-oracle! []
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

(defn execute! []
  (let [source (editor/source)
        mode (or (:language-mode (active-doc)) "text")]
    (swap! state workspace/update-active source)
    (if (contains? #{"my-lisp" "markdown"} mode)
      (if (workspace/native?)
        (-> (workspace/invoke! "evaluate_my_lisp" {:source source :mode mode})
            (.then handle-eval-result!)
            (.catch handle-eval-error!))
        (cond
          (wasm/ready?)
          (-> (wasm/evaluate source mode)
              (.then handle-eval-result!)
              (.catch handle-eval-error!))
          (wasm/failed?)
          (swap! state assoc
                 :output [(case (:language @state)
                            "uk" "Ваш браузер не підтримує WebAssembly (або заблоковано) · code execution unavailable"
                            "de" "Ihr Browser unterstützt WebAssembly nicht (oder blockiert) · code execution unavailable"
                            "Your browser does not support WebAssembly (or blocked) · code execution unavailable")]
                 :error? true)
          :else
          (swap! state assoc
                 :output ["my-lisp WASM engine is loading… · очікуйте завершення завантаження · WASM wird geladen…"]
                 :error? false)))
      (swap! state assoc
             :output [(str (get i18n/programming-language-labels mode)
                           " runtime is not connected yet · runtime ще не підключено · Runtime ist noch nicht verbunden")]
             :error? true))
    (when-not (and (contains? #{"my-lisp" "markdown"} mode) (workspace/native?))
      (render!))))

(defn cycle-programming-language! []
  (when-let [path (:active-path @state)]
    (let [current (or (get-in @state [:documents path :language-mode]) "text")
          next-mode (util/next-value i18n/programming-languages current)]
      (swap! state assoc-in [:documents path :language-mode] next-mode)
      (render!))))

(defn restore-native! []
  (when-let [{:keys [root open-paths active-path]} (workspace/restored)]
    (when (and root (workspace/native?))
      (-> (workspace/invoke! "reopen_workspace" {:path root})
          (.then #(do (swap! state assoc :root % :open-paths [] :active-path nil :documents {})
                      (-> (workspace/invoke! "list_workspace" {}) (.then (fn [nodes] (swap! state assoc :tree (js->clj nodes :keywordize-keys true)) (doseq [path open-paths] (open-file! path)) (when active-path (swap! state assoc :active-path active-path)) (render!))))))
          (.catch (fn [_] (.removeItem js/localStorage workspace/storage-key)))))))
