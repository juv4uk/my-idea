(ns my-idea.workspace
  (:require [clojure.string :as str]
            [my-idea.util :as util]))

(def storage-key "my-idea:workspace")

(defn- tauri-invoke []
  (some-> (aget js/window "__TAURI__") (aget "core") (aget "invoke")))

(defn native? [] (fn? (tauri-invoke)))
(defn invoke! [command args] ((tauri-invoke) command (clj->js args)))

(defn restored []
  (try (js->clj (js/JSON.parse (or (.getItem js/localStorage storage-key) "null")) :keywordize-keys true)
       (catch :default _ nil)))

(defn remember! [model]
  (.setItem js/localStorage storage-key
            (js/JSON.stringify (clj->js (select-keys model [:root :open-paths :active-path])))))

(defn filename [path] (last (str/split path #"/")))

(defn language-mode [path]
  (let [lower (str/lower-case (or path ""))]
    (cond
      (or (str/ends-with? lower ".rs")) "rust"
      (or (str/ends-with? lower ".cljs") (str/ends-with? lower ".cljc") (str/ends-with? lower ".clj")) "clojurescript"
      (or (str/ends-with? lower ".my") (str/ends-with? lower ".lisp")) "my-lisp"
      (or (str/ends-with? lower ".md") (str/ends-with? lower ".markdown")) "markdown"
      (or (str/ends-with? lower ".mermaid") (str/ends-with? lower ".mmd")) "mermaid"
      :else "text")))

(defn browser-path [file]
  (let [relative (.-webkitRelativePath file)]
    (if (str/blank? relative) (.-name file) relative)))

(defonce fs-handles (atom {}))

(def ignored-names #{"node_modules" ".git" "target" "dist" ".cargo" ".rustup"})

(defn- collect-dir-entries [dir-handle]
  (let [result (atom [])
        iter (.values dir-handle)]
    (letfn [(step []
              (-> (.next iter)
                  (.then (fn [item]
                           (if (.-done item)
                             @result
                             (do (swap! result conj (.-value item))
                                 (step)))))))]
      (step))))

(defn scan-dir-tree [dir-handle prefix]
  (-> (collect-dir-entries dir-handle)
      (.then (fn [entries]
               (let [visible (remove #(contains? ignored-names (.-name %)) entries)
                     sorted  (sort-by #(str (if (= (.-kind %) "directory") "0" "1") (.-name %)) visible)
                     promises (map (fn [entry]
                                     (let [entry-path (if (str/blank? prefix)
                                                        (.-name entry)
                                                        (str prefix "/" (.-name entry)))]
                                       (if (= (.-kind entry) "directory")
                                         (-> (scan-dir-tree entry entry-path)
                                             (.then (fn [children]
                                                      {:name (.-name entry) :path entry-path
                                                       :directory true :children children})))
                                         (do (swap! fs-handles assoc entry-path entry)
                                             (js/Promise.resolve
                                               {:name (.-name entry) :path entry-path
                                                :directory false :children []})))))
                                   sorted)]
                 (js/Promise.all (clj->js promises)))))
      (.then (fn [nodes] (vec (js->clj nodes :keywordize-keys true))))))

(defn read-file-from-handle [path]
  (when-let [handle (get @fs-handles path)]
    (-> (if (.-requestPermission handle)
          (-> (.requestPermission handle #js {:mode "read"})
              (.then (fn [status]
                       (if (= status "granted")
                         (-> (.getFile handle) (.then #(.text %)))
                         (js/Promise.reject (js/Error. "Permission denied"))))))
          (-> (.getFile handle) (.then #(.text %))))
        (.catch (fn [err]
                  ;; If handle is stale (after page reload), clear it
                  (swap! fs-handles dissoc path)
                  (js/Promise.reject err))))))

(defn browser-tree [files]
  (->> files
       (map (fn [file]
              (let [path (browser-path file)]
                {:name (filename path) :path path :directory false :children []})))
       (sort-by :path)
       vec))

(defn download! [path contents]
  (let [url (.createObjectURL js/URL (js/Blob. #js [contents] #js {:type "text/plain;charset=utf-8"}))
        link (.createElement js/document "a")]
    (set! (.-href link) url)
    (set! (.-download link) (filename path))
    (.appendChild (.-body js/document) link)
    (.click link)
    (.remove link)
    (js/setTimeout #(.revokeObjectURL js/URL url) 0)))

(defn save-as-browser! [path contents callback]
  (if (exists? js/window.showSaveFilePicker)
    (-> (.showSaveFilePicker js/window
           (clj->js {:suggestedName (filename path)
                     :types [{:description "Source File · Файл коду"
                              :accept {"text/plain" [".my" ".lisp" ".txt" ".rs" ".cljs"]}}]}))
        (.then (fn [handle]
                 (-> (.createWritable handle)
                     (.then (fn [writable]
                              (-> (.write writable contents)
                                  (.then #(.close writable))
                                  (.then #(callback (.-name handle)))))))))
        (.catch (fn [err]
                  (if (= (.-name err) "AbortError")
                    (callback nil)
                    (do (download! path contents)
                        (callback (filename path)))))))
    (do (download! path contents)
        (callback (filename path)))))

(defn update-active [model contents]
  (if-let [path (:active-path model)]
    (-> model
        (assoc-in [:documents path :contents] contents)
        (assoc-in [:documents path :dirty?] (not= contents (get-in model [:documents path :saved]))))
    model))

(defn open-document [model path contents]
  (-> model
      (assoc-in [:documents path] {:contents contents :saved contents :dirty? false :language-mode (language-mode path)})
      (update :open-paths #(vec (distinct (conj (or % []) path))))
      (assoc :active-path path)))

(defn open-browser-workspace [model files contents]
  (let [paths (mapv browser-path files)
        documents (into {} (map (fn [path source]
                                  [path {:contents source :saved source :dirty? false :language-mode (language-mode path)}])
                                paths contents))
        first-path (first paths)
        root (some-> first-path (str/split #"/") first)]
    (assoc model
           :root (or root "Browser workspace")
           :tree (browser-tree files)
           :open-paths (if first-path [first-path] [])
           :active-path first-path
           :documents documents)))

(defn close-document [model path]
  (let [paths (vec (remove #{path} (:open-paths model)))]
    (-> model (assoc :open-paths paths)
        (assoc :active-path (if (= path (:active-path model)) (last paths) (:active-path model))))))

(defn tree-html [nodes]
  (apply str (map (fn [{:keys [name path directory children]}]
                    (if directory
                      (str "<details open><summary>▾ " (util/esc name) "</summary><div>" (tree-html children) "</div></details>")
                      (str "<button class='file' data-path='" (util/esc-attr path) "'>" (util/esc name) "</button>"))) nodes)))
