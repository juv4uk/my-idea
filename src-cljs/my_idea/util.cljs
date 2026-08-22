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

(defn esc-attr [x]
  (-> (esc x)
      ;; single quotes are not escaped by esc — attributes below are
      ;; rendered inside '...' so an apostrophe in a filename/path would
      ;; break out of the attribute (and allow markup injection)
      (str/replace "'" "&#39;")))

(defn next-value [values current]
  (let [index (or (first (keep-indexed #(when (= %2 current) %1) values)) -1)]
    (get values (mod (inc index) (count values)))))
