(ns my-idea.util
  "Small pure helpers with no app-state dependency — kept out of core.cljs
  so it isn't the only place these can live."
  (:require [clojure.string :as str]))

(defn esc [x]
  (-> (str x)
      (str/replace "&" "&amp;")
      (str/replace "<" "&lt;")
      (str/replace ">" "&gt;")
      (str/replace "\"" "&quot;")))

(defn next-value [values current]
  (let [index (or (first (keep-indexed #(when (= %2 current) %1) values)) -1)]
    (get values (mod (inc index) (count values)))))
