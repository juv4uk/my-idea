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

(defn update-active [model contents]
  (if-let [path (:active-path model)]
    (-> model
        (assoc-in [:documents path :contents] contents)
        (assoc-in [:documents path :dirty?] (not= contents (get-in model [:documents path :saved]))))
    model))

(defn open-document [model path contents]
  (-> model
      (assoc-in [:documents path] {:contents contents :saved contents :dirty? false})
      (update :open-paths #(vec (distinct (conj (or % []) path))))
      (assoc :active-path path)))

(defn close-document [model path]
  (let [paths (vec (remove #{path} (:open-paths model)))]
    (-> model (assoc :open-paths paths)
        (assoc :active-path (if (= path (:active-path model)) (last paths) (:active-path model))))))

(defn tree-html [nodes]
  (apply str (map (fn [{:keys [name path directory children]}]
                    (if directory
                      (str "<details open><summary>▾ " name "</summary><div>" (tree-html children) "</div></details>")
                      (str "<button class='file' data-path='" path "'>" name "</button>"))) nodes)))
