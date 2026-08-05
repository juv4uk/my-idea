(ns my-idea.benchmark
  (:require [my-idea.language :as language]))

(def fs (js/require "node:fs"))
(def cases [["arithmetic" "benchmarks/arithmetic.my"]
            ["lists" "benchmarks/lists.my"]
            ["recursion" "benchmarks/recursion.my"]
            ["closures" "benchmarks/closures.my"]])

(defn- read-source [path] (.readFileSync fs path "utf8"))

(defn- measure [iterations operation]
  (dotimes [_ 50] (operation))
  (let [started (.now js/performance)]
    (dotimes [_ iterations] (operation))
    (* 1000000 (/ (- (.now js/performance) started) iterations))))

(defn- report [name nanoseconds]
  (println (str "BENCH_RESULT\tcljs\t" name "\t" (.toFixed nanoseconds 2))))

(defn main []
  (let [iterations (or (some-> (aget js/process "env" "MY_LISP_BENCH_ITERATIONS") js/parseInt) 1000)
        parser-source (read-source "benchmarks/parser.my")]
    (report "parser" (measure iterations #(language/parse-program parser-source)))
    (doseq [[name path] cases]
      (let [source (read-source path)]
        (report name (measure iterations #(language/run-program source)))))))
