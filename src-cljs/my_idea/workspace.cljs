(ns my-idea.workspace
  (:require [clojure.string :as str]))

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
      (or (str/ends-with? lower ".lisp") (str/ends-with? lower ".my")) "my-lisp"
      :else "text")))

(defn browser-path [file]
  (let [relative (.-webkitRelativePath file)]
    (if (str/blank? relative) (.-name file) relative)))

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
                      (str "<details open><summary>▾ " name "</summary><div>" (tree-html children) "</div></details>")
                      (str "<button class='file' data-path='" path "'>" name "</button>"))) nodes)))
